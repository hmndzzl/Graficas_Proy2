use crate::camera::Camera;
use crate::color::Color;
use crate::framebuffer::Framebuffer;
use crate::light::Light;
use crate::ray_intersect::{Intersect, RayIntersect};
use crate::sky::{get_sky_color, noise};
use nalgebra_glm::{dot, normalize, Vec3};
use rayon::prelude::*;
use std::f32::consts::PI;

pub const FOV: f32 = PI / 3.0;
pub const SHADOW_BIAS: f32 = 1e-3;
pub const REFLECTION_BIAS: f32 = 1e-3;
pub const MAX_DEPTH: u32 = 3;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SkyMode {
    Overworld,
    Nether,
}

fn background_color(ray_direction: &Vec3, time_of_day: f32, sky_mode: SkyMode) -> Color {
    match sky_mode {
        SkyMode::Overworld => get_sky_color(ray_direction, time_of_day),
        SkyMode::Nether => {
            // Niebla volcánica oscura: no se reutiliza el cielo azul del mundo normal.
            let horizon_glow = (1.0 - ray_direction.y.abs()).clamp(0.0, 1.0);
            let ceiling_darkness = ((ray_direction.y + 1.0) * 0.5).clamp(0.0, 1.0);
            Color::new(42, 7, 10) * (0.65 + horizon_glow * 0.35)
                + Color::new(35, 8, 5) * (1.0 - ceiling_darkness) * 0.35
        }
    }
}

pub fn reflect(incident: &Vec3, normal: &Vec3) -> Vec3 {
    incident - normal * (2.0 * dot(incident, normal))
}

pub fn refract(incident: &Vec3, normal: &Vec3, eta_t: f32) -> Vec3 {
    let mut cosi = dot(incident, normal).clamp(-1.0, 1.0);
    let mut n = *normal;
    let mut eta_i = 1.0;
    let mut eta_t_local = eta_t;

    if cosi < 0.0 {
        cosi = -cosi;
    } else {
        std::mem::swap(&mut eta_i, &mut eta_t_local);
        n = -*normal;
    }

    let eta = eta_i / eta_t_local;
    let k = 1.0 - eta * eta * (1.0 - cosi * cosi);

    if k < 0.0 {
        reflect(incident, &n)
    } else {
        eta * incident + (eta * cosi - k.sqrt()) * n
    }
}

pub fn cast_shadow(
    intersect: &Intersect,
    light_direction: &Vec3,
    light: &Light,
    objects: &[Box<dyn RayIntersect>],
) -> f32 {
    let shadow_ray_origin = intersect.point + intersect.normal * SHADOW_BIAS;
    let light_distance = (light.position - intersect.point).magnitude();

    let mut shadow_intensity = 0.0;

    for object in objects {
        if let Some(shadow_intersect) = object.ray_intersect(&shadow_ray_origin, light_direction) {
            if shadow_intersect.distance < light_distance {
                let transparency = shadow_intersect.material.albedo[3];
                if transparency > 0.0 {
                    shadow_intensity += 1.0 - transparency;
                } else {
                    return 1.0;
                }
            }
        }
    }

    shadow_intensity.min(1.0)
}

pub fn shade(
    intersect: &Intersect,
    ray_origin: &Vec3,
    lights: &[Light],
    objects: &[Box<dyn RayIntersect>],
) -> Color {
    let base_color = match &intersect.material.texture {
        Some(texture) => {
            let mut u = intersect.u;
            let mut v = intersect.v;
            
            if intersect.material.uv_rotated {
                let temp = u;
                u = v;
                v = 1.0 - temp;
            }

            let u = u * intersect.material.uv_scale.0 + intersect.material.uv_offset.0;
            let v = v * intersect.material.uv_scale.1 + intersect.material.uv_offset.1;
            texture.get_color(u, v)
        }
        None => intersect.material.diffuse,
    };

    if intersect.material.has_emission {
        return base_color * 1.5;
    }

    let view_direction = (ray_origin - intersect.point).normalize();
    let mut total_diffuse = Color::new(0, 0, 0);
    let mut total_specular = Color::new(0, 0, 0);

    for light in lights {
        let light_direction = (light.position - intersect.point).normalize();
        let shadow_intensity = cast_shadow(intersect, &light_direction, light, objects);
        let light_intensity = light.intensity * (1.0 - shadow_intensity);

        let diffuse_intensity = dot(&intersect.normal, &light_direction).max(0.0);
        let diffuse = base_color * (diffuse_intensity * intersect.material.albedo[0] * light_intensity);
        total_diffuse = total_diffuse + diffuse;

        let reflect_direction = reflect(&-light_direction, &intersect.normal);
        let specular_intensity = dot(&view_direction, &reflect_direction)
            .max(0.0)
            .powf(intersect.material.specular);

        let specular = light.color * (specular_intensity * intersect.material.albedo[1] * light_intensity);
        total_specular = total_specular + specular;
    }

    let ambient = base_color * 0.15;
    total_diffuse + total_specular + ambient
}

pub fn cast_ray(
    ray_origin: &Vec3,
    ray_direction: &Vec3,
    objects: &[Box<dyn RayIntersect>],
    lights: &[Light],
    depth: u32,
    time_of_day: f32,
    selected_index: Option<usize>,
    sky_mode: SkyMode,
) -> Color {
    if depth > MAX_DEPTH {
        return background_color(ray_direction, time_of_day, sky_mode);
    }

    let mut closest: Option<Intersect> = None;
    let mut closest_i: Option<usize> = None;

    for (i, object) in objects.iter().enumerate() {
        if let Some(intersect) = object.ray_intersect(ray_origin, ray_direction) {
            if closest
                .as_ref()
                .is_none_or(|current| intersect.distance < current.distance)
            {
                closest = Some(intersect);
                closest_i = Some(i);
            }
        }
    }

    if closest.is_none() {
        return background_color(ray_direction, time_of_day, sky_mode);
    }
    let mut intersect = closest.unwrap();

    // Water Waves (Procedural Normal Perturbation)
    if intersect.material.is_water {
        let nx = noise(intersect.point.x * 3.0 + time_of_day * 10.0, intersect.point.z * 3.0);
        let nz = noise(intersect.point.x * 3.0, intersect.point.z * 3.0 - time_of_day * 10.0);
        intersect.normal = (intersect.normal + Vec3::new(nx - 0.5, 0.0, nz - 0.5) * 0.3).normalize();
    }

    let mut color = shade(&intersect, ray_origin, lights, objects);

    if selected_index.is_some() && closest_i == selected_index {
        color = color + Color::new(80, 80, 80);
    }

    let reflectivity = intersect.material.albedo[2];
    let transparency = intersect.material.albedo[3];

    if reflectivity <= 0.0 && transparency <= 0.0 {
        return color;
    }

    let mut final_color = color * (1.0 - reflectivity - transparency).max(0.0);

    if reflectivity > 0.0 {
        let reflect_direction = reflect(ray_direction, &intersect.normal).normalize();
        let reflect_origin = intersect.point + intersect.normal * REFLECTION_BIAS;
        let reflected = cast_ray(
            &reflect_origin,
            &reflect_direction,
            objects,
            lights,
            depth + 1,
            time_of_day,
            None,
            sky_mode,
        );
        final_color = final_color + reflected * reflectivity;
    }

    if transparency > 0.0 {
        let refract_direction = refract(ray_direction, &intersect.normal, intersect.material.refractive_index).normalize();
        let mut refract_origin = intersect.point - intersect.normal * REFLECTION_BIAS;
        if dot(ray_direction, &intersect.normal) > 0.0 {
            refract_origin = intersect.point + intersect.normal * REFLECTION_BIAS;
        }
        let refracted = cast_ray(
            &refract_origin,
            &refract_direction,
            objects,
            lights,
            depth + 1,
            time_of_day,
            None,
            sky_mode,
        );
        final_color = final_color + refracted * transparency;
    }

    // Atmospheric Depth Fog (Only apply to primary rays to avoid double-fogging transparent/reflective objects)
    if depth == 0 {
        let fog_start = 40.0;
        let fog_end = 100.0;
        let fog_factor = ((intersect.distance - fog_start) / (fog_end - fog_start)).clamp(0.0, 1.0);
        
        if fog_factor > 0.0 {
            let sky_color = background_color(ray_direction, time_of_day, sky_mode);
            final_color = final_color * (1.0 - fog_factor) + sky_color * fog_factor;
        }
    }

    final_color
}

pub fn render(
    framebuffer: &mut Framebuffer,
    objects: &[Box<dyn RayIntersect>],
    camera: &Camera,
    lights: &[Light],
    time_of_day: f32,
    block_size: usize,
    selected_index: Option<usize>,
    sky_mode: SkyMode,
) {
    let width = framebuffer.width;
    let height = framebuffer.height;
    let aspect_ratio = width as f32 / height as f32;
    let perspective_scale = (FOV / 2.0).tan();

    let width_blocks = (width + block_size - 1) / block_size;
    let height_blocks = (height + block_size - 1) / block_size;

    let block_colors: Vec<u32> = (0..width_blocks * height_blocks)
        .into_par_iter()
        .map(|i| {
            let bx = i % width_blocks;
            let by = i / width_blocks;

            let x = bx * block_size;
            let y = by * block_size;

            let screen_x = (2.0 * x as f32) / (width as f32) - 1.0;
            let screen_y = -(2.0 * y as f32) / (height as f32) + 1.0;

            let screen_x = screen_x * aspect_ratio * perspective_scale;
            let screen_y = screen_y * perspective_scale;

            let ray_direction = normalize(&Vec3::new(screen_x, screen_y, -1.0));
            let ray_direction = camera.basis_change(&ray_direction);

            cast_ray(
                &camera.eye,
                &ray_direction,
                objects,
                lights,
                0,
                time_of_day,
                selected_index,
                sky_mode,
            )
            .to_hex()
        })
        .collect();

    for y in 0..height {
        for x in 0..width {
            let bx = x / block_size;
            let by = y / block_size;
            let i = by * width_blocks + bx;
            
            let idx = y * width + x;
            if idx < framebuffer.buffer.len() {
                framebuffer.buffer[idx] = block_colors[i];
            }
        }
    }
}
