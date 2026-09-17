mod camera;
mod color;
mod cube;
mod framebuffer;
mod light;
mod ray_intersect;
mod texture;

use camera::Camera;
use color::Color;
use cube::Cube;
use framebuffer::Framebuffer;
use light::Light;
use minifb::{Key, Window, WindowOptions};
use nalgebra_glm::{Vec3, dot, normalize};
use ray_intersect::{Intersect, Material, RayIntersect};
use std::f32::consts::PI;
use std::sync::Arc;
use std::time::Duration;
use texture::Texture;

const WIDTH: usize = 800;
const HEIGHT: usize = 600;
const BACKGROUND_COLOR: u32 = 0x87CEEB;

const FOV: f32 = PI / 3.0;

const ROTATION_SPEED: f32 = PI / 60.0;

const SHADOW_BIAS: f32 = 1e-3;
const REFLECTION_BIAS: f32 = 1e-3;

const MAX_DEPTH: u32 = 3;

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
    light: &Light,
    objects: &[Box<dyn RayIntersect>],
) -> Color {
    let base_color = match &intersect.material.texture {
        Some(texture) => {
            let u = intersect.u * intersect.material.uv_scale.0 + intersect.material.uv_offset.0;
            let v = intersect.v * intersect.material.uv_scale.1 + intersect.material.uv_offset.1;
            texture.get_color(u, v)
        }
        None => intersect.material.diffuse,
    };

    if intersect.material.has_emission {
        return base_color * 1.5;
    }

    let light_direction = (light.position - intersect.point).normalize();
    let view_direction = (ray_origin - intersect.point).normalize();

    let shadow_intensity = cast_shadow(intersect, &light_direction, light, objects);
    let light_intensity = light.intensity * (1.0 - shadow_intensity);

    let diffuse_intensity = dot(&intersect.normal, &light_direction).max(0.0);
    let diffuse = base_color * (diffuse_intensity * intersect.material.albedo[0] * light_intensity);

    let reflect_direction = reflect(&-light_direction, &intersect.normal);
    let specular_intensity = dot(&view_direction, &reflect_direction)
        .max(0.0)
        .powf(intersect.material.specular);

    let specular =
        light.color * (specular_intensity * intersect.material.albedo[1] * light_intensity);

    let ambient = base_color * 0.15;

    diffuse + specular + ambient
}

pub fn cast_ray(
    ray_origin: &Vec3,
    ray_direction: &Vec3,
    objects: &[Box<dyn RayIntersect>],
    light: &Light,
    depth: u32,
) -> Color {
    if depth > MAX_DEPTH {
        return Color::from_hex(BACKGROUND_COLOR);
    }

    let mut closest: Option<Intersect> = None;

    for object in objects {
        if let Some(intersect) = object.ray_intersect(ray_origin, ray_direction) {
            if closest
                .as_ref()
                .is_none_or(|current| intersect.distance < current.distance)
            {
                closest = Some(intersect);
            }
        }
    }

    let Some(intersect) = closest else {
        return Color::from_hex(BACKGROUND_COLOR);
    };

    let color = shade(&intersect, ray_origin, light, objects);

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
            light,
            depth + 1,
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
            light,
            depth + 1,
        );
        final_color = final_color + refracted * transparency;
    }

    final_color
}

pub fn render(
    framebuffer: &mut Framebuffer,
    objects: &[Box<dyn RayIntersect>],
    camera: &Camera,
    light: &Light,
) {
    let width = framebuffer.width as f32;
    let height = framebuffer.height as f32;
    let aspect_ratio = width / height;

    let perspective_scale = (FOV / 2.0).tan();

    for y in 0..framebuffer.height {
        for x in 0..framebuffer.width {
            let screen_x = (2.0 * x as f32) / width - 1.0;
            let screen_y = -(2.0 * y as f32) / height + 1.0;

            let screen_x = screen_x * aspect_ratio * perspective_scale;
            let screen_y = screen_y * perspective_scale;

            let ray_direction = normalize(&Vec3::new(screen_x, screen_y, -1.0));
            let ray_direction = camera.basis_change(&ray_direction);

            framebuffer.set_current_color(
                cast_ray(&camera.eye, &ray_direction, objects, light, 0).to_hex(),
            );
            framebuffer.point(x, y);
        }
    }
}

fn main() {
    let frame_delay = Duration::from_millis(16);

    let mut framebuffer = Framebuffer::new(WIDTH, HEIGHT);

    let mut window = Window::new(
        "Minecraft Raytracer Diorama",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .unwrap();

    let texture_atlas = Arc::new(Texture::new("assets/textures.png"));
    let uv_scale = (1.0 / 16.0, 1.0 / 16.0);

    let grass_side = Material::new(Color::new(255, 255, 255), 10.0, [0.8, 0.1, 0.0, 0.0])
        .with_texture(Arc::clone(&texture_atlas))
        .with_uv(uv_scale, (3.0 / 16.0, 15.0 / 16.0));
    let grass_top = Material::new(Color::new(255, 255, 255), 10.0, [0.8, 0.1, 0.0, 0.0])
        .with_texture(Arc::clone(&texture_atlas))
        .with_uv(uv_scale, (0.0, 15.0 / 16.0));

    // Stone: (1, 0)
    let stone_mat = Material::new(Color::new(255, 255, 255), 15.0, [0.8, 0.1, 0.0, 0.0])
        .with_texture(Arc::clone(&texture_atlas))
        .with_uv(uv_scale, (1.0 / 16.0, 15.0 / 16.0));

    // Gold Block: (7, 1)
    let gold_mat = Material::new(Color::new(255, 255, 255), 80.0, [0.8, 0.5, 0.2, 0.0])
        .with_texture(Arc::clone(&texture_atlas))
        .with_uv(uv_scale, (7.0 / 16.0, 14.0 / 16.0));

    // Wood Log side: (4, 1)
    let wood_mat = Material::new(Color::new(255, 255, 255), 10.0, [0.8, 0.1, 0.0, 0.0])
        .with_texture(Arc::clone(&texture_atlas))
        .with_uv(uv_scale, (4.0 / 16.0, 14.0 / 16.0));
    // Wood Log top: (5, 1)
    let wood_top = Material::new(Color::new(255, 255, 255), 10.0, [0.8, 0.1, 0.0, 0.0])
        .with_texture(Arc::clone(&texture_atlas))
        .with_uv(uv_scale, (5.0 / 16.0, 14.0 / 16.0));

    // Lava (Emissive)
    let lava_mat = Material::new(Color::new(255, 120, 0), 0.0, [1.0, 0.0, 0.0, 0.0])
        .with_emission(true);

    // Glass (Transparent/Refractive)
    let glass_mat = Material::new(Color::new(200, 220, 255), 50.0, [0.1, 0.4, 0.1, 0.8])
        .with_refractive_index(1.5);
    
    // Water (Transparent/Refractive)
    let water_mat = Material::new(Color::new(50, 100, 255), 40.0, [0.2, 0.3, 0.1, 0.6])
        .with_refractive_index(1.33);

    let objects: Vec<Box<dyn RayIntersect>> = vec![
        // Central block (Gold / Reflective)
        Box::new(Cube::new(
            Vec3::new(-0.5, 0.0, -0.5),
            Vec3::new(0.5, 1.0, 0.5),
            gold_mat,
        )),
        // Block to the left (Grass)
        Box::new(
            Cube::new(
                Vec3::new(-1.6, 0.0, -0.5),
                Vec3::new(-0.6, 1.0, 0.5),
                grass_side,
            )
            .with_top_material(grass_top),
        ),
        // Block to the right (Wood)
        Box::new(
            Cube::new(
                Vec3::new(0.6, 0.0, -0.5),
                Vec3::new(1.6, 1.0, 0.5),
                wood_mat,
            )
            .with_top_material(wood_top),
        ),
        // Lava block (Emissive)
        Box::new(Cube::new(
            Vec3::new(1.7, 0.0, -0.5),
            Vec3::new(2.7, 1.0, 0.5),
            lava_mat,
        )),
        // Glass block (Refractive)
        Box::new(Cube::new(
            Vec3::new(-0.5, 1.0, -0.5),
            Vec3::new(0.5, 2.0, 0.5),
            glass_mat,
        )),
        // Water block
        Box::new(Cube::new(
            Vec3::new(-2.7, 0.0, -0.5),
            Vec3::new(-1.7, 1.0, 0.5),
            water_mat,
        )),
        // Floor platform (Stone)
        Box::new(Cube::new(
            Vec3::new(-4.0, -1.0, -4.0),
            Vec3::new(4.0, 0.0, 4.0),
            stone_mat,
        )),
    ];

    let light = Light::new(Vec3::new(-6.0, 6.0, 8.0), Color::new(255, 255, 255), 1.5);

    let mut camera = Camera::new(
        Vec3::new(0.0, 2.0, 5.5),
        Vec3::new(0.0, 0.3, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
    );

    let mut camera_moved = true;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let orbit = [
            (Key::Left, ROTATION_SPEED, 0.0),
            (Key::Right, -ROTATION_SPEED, 0.0),
            (Key::Up, 0.0, -ROTATION_SPEED),
            (Key::Down, 0.0, ROTATION_SPEED),
        ];

        for (key, delta_yaw, delta_pitch) in orbit {
            if window.is_key_down(key) {
                camera.orbit(delta_yaw, delta_pitch);
                camera_moved = true;
            }
        }

        // Zoom with W / S keys
        if window.is_key_down(Key::W) {
            camera.zoom(0.96);
            camera_moved = true;
        }
        if window.is_key_down(Key::S) {
            camera.zoom(1.04);
            camera_moved = true;
        }

        // Zoom with mouse scroll wheel
        if let Some((_, scroll_y)) = window.get_scroll_wheel() {
            if scroll_y > 0.0 {
                camera.zoom(0.92);
                camera_moved = true;
            } else if scroll_y < 0.0 {
                camera.zoom(1.08);
                camera_moved = true;
            }
        }

        if camera_moved {
            render(&mut framebuffer, &objects, &camera, &light);
            camera_moved = false;
        }

        window
            .update_with_buffer(&framebuffer.buffer, WIDTH, HEIGHT)
            .unwrap();

        std::thread::sleep(frame_delay);
    }
}
