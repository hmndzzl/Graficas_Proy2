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
use rayon::prelude::*;

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

pub fn get_sky_color(ray_direction: &Vec3, time_of_day: f32) -> Color {
    let day_zenith = Color::new(80, 150, 255);
    let day_horizon = Color::new(180, 220, 255);
    
    let night_zenith = Color::new(2, 2, 10);
    let night_horizon = Color::new(10, 15, 30);
    
    let sunset_horizon = Color::new(255, 120, 50);

    let t_y = ray_direction.y.max(0.0);
    let sun_height = (time_of_day * std::f32::consts::PI * 2.0 - std::f32::consts::PI / 2.0).sin();
    
    let day_factor = ((sun_height + 0.2) * 2.0).clamp(0.0, 1.0);
    let sunset_factor = (1.0 - sun_height.abs() * 3.0).clamp(0.0, 1.0);
    
    let current_zenith = night_zenith * (1.0 - day_factor) + day_zenith * day_factor;
    let base_horizon = night_horizon * (1.0 - day_factor) + day_horizon * day_factor;
    let current_horizon = base_horizon * (1.0 - sunset_factor) + sunset_horizon * sunset_factor;
    
    let mut sky = current_horizon * (1.0 - t_y) + current_zenith * t_y;

    // Stars
    if day_factor < 0.2 {
        let qx = (ray_direction.x * 500.0).round();
        let qy = (ray_direction.y * 500.0).round();
        let qz = (ray_direction.z * 500.0).round();
        
        let seed = qx * 12.9898 + qy * 78.233 + qz * 37.719;
        let hash = (seed.sin() * 43758.5453).fract().abs();
        
        if hash > 0.995 {
            let star_brightness = (hash - 0.995) * 200.0;
            let visibility = (1.0 - day_factor * 5.0).clamp(0.0, 1.0);
            sky = sky + Color::new(255, 255, 255) * (star_brightness * visibility);
        }
    }

    sky
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
) -> Color {
    if depth > MAX_DEPTH {
        return get_sky_color(ray_direction, time_of_day);
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

    let Some(intersect) = closest else {
        return get_sky_color(ray_direction, time_of_day);
    };

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
        );
        final_color = final_color + refracted * transparency;
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

            cast_ray(&camera.eye, &ray_direction, objects, lights, 0, time_of_day, selected_index).to_hex()
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

fn blend(bg: u32, fg: u32, alpha: f32) -> u32 {
    let r1 = ((bg >> 16) & 0xFF) as f32;
    let g1 = ((bg >> 8) & 0xFF) as f32;
    let b1 = (bg & 0xFF) as f32;

    let r2 = ((fg >> 16) & 0xFF) as f32;
    let g2 = ((fg >> 8) & 0xFF) as f32;
    let b2 = (fg & 0xFF) as f32;

    let r = (r1 * (1.0 - alpha) + r2 * alpha) as u32;
    let g = (g1 * (1.0 - alpha) + g2 * alpha) as u32;
    let b = (b1 * (1.0 - alpha) + b2 * alpha) as u32;

    (r << 16) | (g << 8) | b
}

pub fn draw_ui(framebuffer: &mut Framebuffer, inventory: &[(&str, Material)], active_index: usize) {
    let width = framebuffer.width;
    let height = framebuffer.height;
    
    // Crosshair
    let cx = width / 2;
    let cy = height / 2;
    let crosshair_color = 0xDDDDDD;
    for i in 0..8 {
        if cx + i < width { framebuffer.buffer[cy * width + (cx + i)] = blend(framebuffer.buffer[cy * width + (cx + i)], crosshair_color, 0.8); }
        if cx >= i { framebuffer.buffer[cy * width + (cx - i)] = blend(framebuffer.buffer[cy * width + (cx - i)], crosshair_color, 0.8); }
        if cy + i < height { framebuffer.buffer[(cy + i) * width + cx] = blend(framebuffer.buffer[(cy + i) * width + cx], crosshair_color, 0.8); }
        if cy >= i { framebuffer.buffer[(cy - i) * width + cx] = blend(framebuffer.buffer[(cy - i) * width + cx], crosshair_color, 0.8); }
    }
    
    // Hotbar (Minecraft Style)
    let slot_size = 40;
    let num_slots = 9;
    let total_width = num_slots * slot_size;
    let start_x = (width - total_width) / 2;
    let start_y = height - slot_size - 20;

    // Draw Hotbar Background and Slots
    for i in 0..num_slots {
        let x0 = start_x + i * slot_size;
        let y0 = start_y;
        
        for bx in 0..slot_size {
            for by in 0..slot_size {
                let px = x0 + bx;
                let py = y0 + by;
                if px >= width || py >= height { continue; }

                let bg = framebuffer.buffer[py * width + px];
                let mut fg = 0x8B8B8B;
                let mut alpha = 0.5;

                // Slot borders (top/left are dark, bottom/right are white in minecraft? actually top/left are lighter, bottom/right darker)
                if bx < 2 || by < 2 {
                    fg = 0x373737;
                    alpha = 0.8;
                } else if bx >= slot_size - 2 || by >= slot_size - 2 {
                    fg = 0xFFFFFF;
                    alpha = 0.4;
                }

                framebuffer.buffer[py * width + px] = blend(bg, fg, alpha);
            }
        }
    }

    // Draw Items
    for (i, (_, mat)) in inventory.iter().enumerate() {
        let item_size = slot_size - 12;
        let x0 = start_x + i * slot_size + 6;
        let y0 = start_y + 6;

        if let Some(texture) = &mat.texture {
            for bx in 0..item_size {
                for by in 0..item_size {
                    let px = x0 + bx;
                    let py = y0 + by;
                    if px >= width || py >= height { continue; }

                    let u = bx as f32 / item_size as f32;
                    let v = 1.0 - (by as f32 / item_size as f32);
                    
                    let mut u_map = u;
                    let mut v_map = v;
                    if mat.uv_rotated {
                        u_map = v;
                        v_map = 1.0 - u;
                    }
                    
                    let final_u = u_map * mat.uv_scale.0 + mat.uv_offset.0;
                    let final_v = v_map * mat.uv_scale.1 + mat.uv_offset.1;
                    
                    framebuffer.buffer[py * width + px] = texture.get_color(final_u, final_v).to_hex();
                }
            }
        } else {
            for bx in 0..item_size {
                for by in 0..item_size {
                    let px = x0 + bx;
                    let py = y0 + by;
                    if px < width && py < height {
                        framebuffer.buffer[py * width + px] = mat.diffuse.to_hex();
                    }
                }
            }
        }
    }

    // Draw Active Selector Overlay
    let ax0 = start_x + active_index * slot_size;
    let ay0 = start_y;
    for bx in 0..slot_size {
        for by in 0..slot_size {
            let px = ax0 + bx;
            let py = ay0 + by;
            if px >= width || py >= height { continue; }
            
            // 3px white border around the active slot
            if bx < 3 || by < 3 || bx >= slot_size - 3 || by >= slot_size - 3 {
                framebuffer.buffer[py * width + px] = 0xFFFFFF; // White border
            }
        }
    }
}

pub fn build_diorama(texture_atlas: Arc<Texture>) -> Vec<Box<dyn RayIntersect>> {
    let mut objects: Vec<Box<dyn RayIntersect>> = Vec::new();
    let uv_scale = (1.0 / 16.0, 1.0 / 16.0);

    let grass_side = Material::new(Color::new(255, 255, 255), 10.0, [0.8, 0.1, 0.0, 0.0])
        .with_texture(Arc::clone(&texture_atlas))
        .with_uv(uv_scale, (3.0 / 16.0, 15.0 / 16.0));
    let grass_top = Material::new(Color::new(255, 255, 255), 10.0, [0.8, 0.1, 0.0, 0.0])
        .with_texture(Arc::clone(&texture_atlas))
        .with_uv(uv_scale, (0.0, 15.0 / 16.0));
    let stone_mat = Material::new(Color::new(255, 255, 255), 15.0, [0.8, 0.1, 0.0, 0.0])
        .with_texture(Arc::clone(&texture_atlas))
        .with_uv(uv_scale, (1.0 / 16.0, 15.0 / 16.0));
    let gold_mat = Material::new(Color::new(255, 255, 255), 80.0, [0.8, 0.5, 0.2, 0.0])
        .with_texture(Arc::clone(&texture_atlas))
        .with_uv(uv_scale, (7.0 / 16.0, 14.0 / 16.0));
    let diamond_ore_mat = Material::new(Color::new(255, 255, 255), 20.0, [0.8, 0.2, 0.0, 0.0])
        .with_texture(Arc::clone(&texture_atlas))
        .with_uv(uv_scale, (2.0 / 16.0, 12.0 / 16.0)); // (2, 3)
    
    let wood_mat = Material::new(Color::new(255, 255, 255), 10.0, [0.8, 0.1, 0.0, 0.0])
        .with_texture(Arc::clone(&texture_atlas))
        .with_uv(uv_scale, (4.0 / 16.0, 14.0 / 16.0));
    let wood_top = Material::new(Color::new(255, 255, 255), 10.0, [0.8, 0.1, 0.0, 0.0])
        .with_texture(Arc::clone(&texture_atlas))
        .with_uv(uv_scale, (5.0 / 16.0, 14.0 / 16.0));
    let planks_mat = Material::new(Color::new(255, 255, 255), 10.0, [0.8, 0.1, 0.0, 0.0])
        .with_texture(Arc::clone(&texture_atlas))
        .with_uv(uv_scale, (4.0 / 16.0, 15.0 / 16.0)); // (4, 0)
    let leaves_mat = Material::new(Color::new(255, 255, 255), 5.0, [0.8, 0.0, 0.0, 0.0])
        .with_texture(Arc::clone(&texture_atlas))
        .with_uv(uv_scale, (4.0 / 16.0, 12.0 / 16.0)); // (4, 3)

    let crafting_top = Material::new(Color::new(255, 255, 255), 10.0, [0.8, 0.1, 0.0, 0.0])
        .with_texture(Arc::clone(&texture_atlas))
        .with_uv(uv_scale, (11.0 / 16.0, 13.0 / 16.0)); // (11, 2)
    let crafting_front = Material::new(Color::new(255, 255, 255), 10.0, [0.8, 0.1, 0.0, 0.0])
        .with_texture(Arc::clone(&texture_atlas))
        .with_uv(uv_scale, (11.0 / 16.0, 12.0 / 16.0)); // (11, 3)
    let crafting_side = Material::new(Color::new(255, 255, 255), 10.0, [0.8, 0.1, 0.0, 0.0])
        .with_texture(Arc::clone(&texture_atlas))
        .with_uv(uv_scale, (12.0 / 16.0, 12.0 / 16.0)); // (12, 3)

    let furnace_top = Material::new(Color::new(255, 255, 255), 10.0, [0.8, 0.1, 0.0, 0.0])
        .with_texture(Arc::clone(&texture_atlas))
        .with_uv(uv_scale, (14.0 / 16.0, 12.0 / 16.0)); // (14, 3)
    let furnace_front = Material::new(Color::new(255, 255, 255), 10.0, [0.8, 0.1, 0.0, 0.0])
        .with_texture(Arc::clone(&texture_atlas))
        .with_uv(uv_scale, (13.0 / 16.0, 12.0 / 16.0)); // (13, 3) front on
    let furnace_side = Material::new(Color::new(255, 255, 255), 10.0, [0.8, 0.1, 0.0, 0.0])
        .with_texture(Arc::clone(&texture_atlas))
        .with_uv(uv_scale, (13.0 / 16.0, 13.0 / 16.0)); // (13, 2)

    let lava_mat = Material::new(Color::new(255, 120, 0), 0.0, [1.0, 0.0, 0.0, 0.0])
        .with_texture(Arc::clone(&texture_atlas))
        .with_uv(uv_scale, (15.0 / 16.0, 0.0)) // (15, 15)
        .with_emission(true);
    let glass_mat = Material::new(Color::new(200, 220, 255), 50.0, [0.1, 0.4, 0.1, 0.8])
        .with_refractive_index(1.5);
    let water_mat = Material::new(Color::new(50, 100, 255), 40.0, [0.2, 0.3, 0.1, 0.6])
        .with_refractive_index(1.33);

    let bed_foot_top = Material::new(Color::new(255, 255, 255), 5.0, [0.8, 0.0, 0.0, 0.0])
        .with_texture(Arc::clone(&texture_atlas))
        .with_uv(uv_scale, (6.0 / 16.0, 7.0 / 16.0)) // (6, 8)
        .with_uv_rotated(true);
    let bed_head_top = Material::new(Color::new(255, 255, 255), 5.0, [0.8, 0.0, 0.0, 0.0])
        .with_texture(Arc::clone(&texture_atlas))
        .with_uv(uv_scale, (7.0 / 16.0, 7.0 / 16.0)) // (7, 8)
        .with_uv_rotated(true);
    let bed_foot_front = Material::new(Color::new(255, 255, 255), 5.0, [0.8, 0.0, 0.0, 0.0])
        .with_texture(Arc::clone(&texture_atlas))
        .with_uv(uv_scale, (5.0 / 16.0, 6.0 / 16.0)); // (5, 9)
    let bed_head_back = Material::new(Color::new(255, 255, 255), 5.0, [0.8, 0.0, 0.0, 0.0])
        .with_texture(Arc::clone(&texture_atlas))
        .with_uv(uv_scale, (8.0 / 16.0, 6.0 / 16.0)); // (8, 9)
    let bed_foot_side = Material::new(Color::new(255, 255, 255), 5.0, [0.8, 0.0, 0.0, 0.0])
        .with_texture(Arc::clone(&texture_atlas))
        .with_uv(uv_scale, (6.0 / 16.0, 6.0 / 16.0)); // (6, 9)
    let bed_head_side = Material::new(Color::new(255, 255, 255), 5.0, [0.8, 0.0, 0.0, 0.0])
        .with_texture(Arc::clone(&texture_atlas))
        .with_uv(uv_scale, (7.0 / 16.0, 6.0 / 16.0)); // (7, 9)

    // Island
    let radius = 6.0;
    for x in -6..=6 {
        for z in -6..=6 {
            for y in -5..=0 {
                let fx = x as f32;
                let fz = z as f32;
                let fy = y as f32;
                
                let dist = (fx * fx + fz * fz).sqrt();
                let current_radius = radius + (fy * 1.2); 

                if dist <= current_radius {
                    if x > 0 && z > 0 { continue; } // Cut for the mine

                    let is_lake = y == 0 && x >= -5 && x <= -3 && z >= -2 && z <= 0;

                    if is_lake {
                        objects.push(Box::new(Cube::new(
                            Vec3::new(fx - 0.5, fy - 0.6, fz - 0.5),
                            Vec3::new(fx + 0.5, fy - 0.1, fz + 0.5),
                            water_mat.clone(),
                        )));
                        continue;
                    }

                    let mut mat = stone_mat.clone();
                    let mut top_mat = None;

                    if y == 0 {
                        mat = grass_side.clone();
                        top_mat = Some(grass_top.clone());
                    } else if y < -1 && (x * 7 + z * 3 + y * 11) % 17 == 0 {
                        mat = diamond_ore_mat.clone();
                    }
                    
                    let mut cube = Cube::new(
                        Vec3::new(fx - 0.5, fy - 0.5, fz - 0.5),
                        Vec3::new(fx + 0.5, fy + 0.5, fz + 0.5),
                        mat,
                    );
                    if let Some(t) = top_mat {
                        cube = cube.with_top_material(t);
                    }
                    objects.push(Box::new(cube));
                }
            }
        }
    }

    // Lava pool inside the cut
    for x in 1..=3 {
        for z in 1..=3 {
            objects.push(Box::new(Cube::new(
                Vec3::new(x as f32 - 0.5, -4.5, z as f32 - 0.5),
                Vec3::new(x as f32 + 0.5, -3.7, z as f32 + 0.5),
                lava_mat.clone(),
            )));
        }
    }

    // Natural Tree at (-3, 1, 2)
    let tx = -3.0; let ty = 1.0; let tz = 2.0;
    // Trunk
    for i in 0..4 {
        objects.push(Box::new(Cube::new(
            Vec3::new(tx - 0.5, ty + i as f32 - 0.5, tz - 0.5),
            Vec3::new(tx + 0.5, ty + i as f32 + 0.5, tz + 0.5),
            wood_mat.clone(),
        ).with_top_material(wood_top.clone())));
    }
    // Leaves
    for lx in -1i32..=1 {
        for lz in -1i32..=1 {
            for ly in 3..=4 {
                if lx == 0 && lz == 0 && ly == 3 { continue; } // Trunk
                if ly == 4 && lx.abs() == 1 && lz.abs() == 1 { continue; } // Make canopy round
                
                objects.push(Box::new(Cube::new(
                    Vec3::new(tx + lx as f32 - 0.5, ty + ly as f32 - 0.5, tz + lz as f32 - 0.5),
                    Vec3::new(tx + lx as f32 + 0.5, ty + ly as f32 + 0.5, tz + lz as f32 + 0.5),
                    leaves_mat.clone(),
                )));
            }
        }
    }

    // Cabin at (-1, 1, -4)
    // Walls
    for x in -2i32..=2 {
        for z in -5i32..=-2 {
            if x > -2 && x < 2 && z > -5 && z < -2 { continue; } // Empty inside
            for y in 1..=3 {
                // Door
                if x == 0 && z == -2 && (y == 1 || y == 2) { continue; }
                
                // Windows
                if x.abs() == 2 && z == -3 && y == 2 {
                    objects.push(Box::new(Cube::new(
                        Vec3::new(x as f32 - 0.5, y as f32 - 0.5, z as f32 - 0.5),
                        Vec3::new(x as f32 + 0.5, y as f32 + 0.5, z as f32 + 0.5),
                        glass_mat.clone(),
                    )));
                    continue;
                }

                let is_corner = x.abs() == 2 && (z == -5 || z == -2);
                let mat = if is_corner { wood_mat.clone() } else { planks_mat.clone() };
                
                let mut cube = Cube::new(
                    Vec3::new(x as f32 - 0.5, y as f32 - 0.5, z as f32 - 0.5),
                    Vec3::new(x as f32 + 0.5, y as f32 + 0.5, z as f32 + 0.5),
                    mat,
                );
                if is_corner { cube = cube.with_top_material(wood_top.clone()); }
                objects.push(Box::new(cube));
            }
        }
    }
    // Roof (Planks)
    for x in -2..=2 {
        for z in -5..=-2 {
            objects.push(Box::new(Cube::new(
                Vec3::new(x as f32 - 0.5, 4.0 - 0.5, z as f32 - 0.5),
                Vec3::new(x as f32 + 0.5, 4.0 + 0.5, z as f32 + 0.5),
                planks_mat.clone(),
            )));
        }
    }

    // Interior: Crafting Table, Furnace, Bed
    objects.push(Box::new(Cube::new(
        Vec3::new(-1.0 - 0.5, 1.0 - 0.5, -4.0 - 0.5),
        Vec3::new(-1.0 + 0.5, 1.0 + 0.5, -4.0 + 0.5),
        crafting_side.clone(),
    )
    .with_top_material(crafting_top.clone())
    .with_front_material(crafting_front.clone()) // Faces +Z (door)
    .with_back_material(crafting_front.clone())));

    objects.push(Box::new(Cube::new(
        Vec3::new(1.0 - 0.5, 1.0 - 0.5, -4.0 - 0.5),
        Vec3::new(1.0 + 0.5, 1.0 + 0.5, -4.0 + 0.5),
        furnace_side.clone(),
    )
    .with_top_material(furnace_top.clone())
    .with_front_material(furnace_front.clone())));

    // Bed (Pillow at -4, mattress at -3)
    objects.push(Box::new(Cube::new(
        Vec3::new(0.0 - 0.5, 1.0 - 0.5, -4.0 - 0.5), // Headboard side (-4 in Z)
        Vec3::new(0.0 + 0.5, 1.0 - 0.1, -4.0 + 0.5),
        planks_mat.clone(),
    )
    .with_top_material(bed_head_top.clone())
    .with_back_material(bed_head_back.clone()) // Back is -Z
    .with_left_material(bed_head_side.clone())
    .with_right_material(bed_head_side.clone())
    .with_front_material(bed_head_side.clone()))); // Connects to foot

    objects.push(Box::new(Cube::new(
        Vec3::new(0.0 - 0.5, 1.0 - 0.5, -3.0 - 0.5), // Foot side (-3 in Z)
        Vec3::new(0.0 + 0.5, 1.0 - 0.1, -3.0 + 0.5),
        planks_mat.clone(),
    )
    .with_top_material(bed_foot_top.clone())
    .with_front_material(bed_foot_front.clone()) // Front is +Z
    .with_left_material(bed_foot_side.clone())
    .with_right_material(bed_foot_side.clone())
    .with_back_material(bed_foot_side.clone()))); // Connects to head

    // Gold decorations outside the cabin
    objects.push(Box::new(Cube::new(
        Vec3::new(-2.0 - 0.5, 1.0 - 0.5, -1.0 - 0.5),
        Vec3::new(-2.0 + 0.5, 1.0 + 0.5, -1.0 + 0.5),
        gold_mat.clone(),
    )));
    objects.push(Box::new(Cube::new(
        Vec3::new(2.0 - 0.5, 1.0 - 0.5, -1.0 - 0.5),
        Vec3::new(2.0 + 0.5, 1.0 + 0.5, -1.0 + 0.5),
        gold_mat.clone(),
    )));

    objects
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

    let mut objects = build_diorama(Arc::clone(&texture_atlas));

    // Inventory definition
    let uv_scale = (1.0 / 16.0, 1.0 / 16.0);
    let grass_mat = Material::new(Color::new(255, 255, 255), 10.0, [0.8, 0.1, 0.0, 0.0])
        .with_texture(Arc::clone(&texture_atlas))
        .with_uv(uv_scale, (3.0 / 16.0, 15.0 / 16.0));
    let grass_top = Material::new(Color::new(255, 255, 255), 10.0, [0.8, 0.1, 0.0, 0.0])
        .with_texture(Arc::clone(&texture_atlas))
        .with_uv(uv_scale, (0.0, 15.0 / 16.0));
    let planks_mat = Material::new(Color::new(255, 255, 255), 10.0, [0.8, 0.1, 0.0, 0.0])
        .with_texture(Arc::clone(&texture_atlas))
        .with_uv(uv_scale, (4.0 / 16.0, 15.0 / 16.0));
    let stone_mat = Material::new(Color::new(255, 255, 255), 15.0, [0.8, 0.1, 0.0, 0.0])
        .with_texture(Arc::clone(&texture_atlas))
        .with_uv(uv_scale, (1.0 / 16.0, 15.0 / 16.0));
    let glass_mat = Material::new(Color::new(200, 220, 255), 50.0, [0.1, 0.4, 0.1, 0.8])
        .with_refractive_index(1.5);
    let leaves_mat = Material::new(Color::new(255, 255, 255), 5.0, [0.8, 0.1, 0.0, 0.0])
        .with_texture(Arc::clone(&texture_atlas))
        .with_uv(uv_scale, (4.0 / 16.0, 12.0 / 16.0));

    let inventory = vec![
        ("Césped", grass_mat),
        ("Tablas", planks_mat),
        ("Piedra", stone_mat),
        ("Hojas", leaves_mat),
        ("Cristal", glass_mat),
    ];
    let mut active_block_index = 0;

    let mut time_of_day = 0.5; // Noon

    let mut camera = Camera::new(
        Vec3::new(0.0, 2.0, 5.5),
        Vec3::new(0.0, 0.3, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
    );

    let mut camera_state = 0; // 0 = Moving (Render Low Res), 1 = Stopped (Render High Res), 2 = Done

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let mut moved = false;
        
        if window.is_key_pressed(Key::D, minifb::KeyRepeat::No) {
            time_of_day = 0.5; // Day
            moved = true;
        }
        if window.is_key_pressed(Key::N, minifb::KeyRepeat::No) {
            time_of_day = 0.0; // Night
            moved = true;
        }
        if window.is_key_down(Key::T) {
            time_of_day += 0.01;
            if time_of_day >= 1.0 { time_of_day -= 1.0; }
            moved = true;
        }

        if window.is_key_pressed(Key::Key1, minifb::KeyRepeat::No) { active_block_index = 0; moved = true; }
        if window.is_key_pressed(Key::Key2, minifb::KeyRepeat::No) { active_block_index = 1; moved = true; }
        if window.is_key_pressed(Key::Key3, minifb::KeyRepeat::No) { active_block_index = 2; moved = true; }
        if window.is_key_pressed(Key::Key4, minifb::KeyRepeat::No) { active_block_index = 3; moved = true; }
        if window.is_key_pressed(Key::Key5, minifb::KeyRepeat::No) { active_block_index = 4; moved = true; }

        // Block Selection Raycasting
        let center_ray = normalize(&Vec3::new(0.0, 0.0, -1.0));
        let center_ray = camera.basis_change(&center_ray);
        let mut selected_block: Option<(usize, Intersect)> = None;
        for (i, object) in objects.iter().enumerate() {
            if let Some(intersect) = object.ray_intersect(&camera.eye, &center_ray) {
                if intersect.distance < 10.0 {
                    if selected_block.as_ref().is_none_or(|(_, current)| intersect.distance < current.distance) {
                        selected_block = Some((i, intersect));
                    }
                }
            }
        }

        if window.is_key_pressed(Key::Space, minifb::KeyRepeat::No) {
            if let Some((_, intersect)) = &selected_block {
                let hit_center_x = (intersect.point.x - intersect.normal.x * 0.01).round();
                let hit_center_y = (intersect.point.y - intersect.normal.y * 0.01).round();
                let hit_center_z = (intersect.point.z - intersect.normal.z * 0.01).round();
                let new_center = Vec3::new(hit_center_x, hit_center_y, hit_center_z) + intersect.normal;
                
                let active_name = inventory[active_block_index].0;
                let active_mat = inventory[active_block_index].1.clone();
                let mut cube = Cube::new(
                    new_center - Vec3::new(0.5, 0.5, 0.5),
                    new_center + Vec3::new(0.5, 0.5, 0.5),
                    active_mat,
                );

                if active_name == "Césped" {
                    let dirt = Material::new(Color::new(255, 255, 255), 10.0, [0.8, 0.1, 0.0, 0.0])
                        .with_texture(Arc::clone(&texture_atlas))
                        .with_uv(uv_scale, (2.0 / 16.0, 15.0 / 16.0));
                        
                    cube = cube.with_top_material(grass_top.clone())
                               .with_bottom_material(dirt);
                }

                objects.push(Box::new(cube));
                moved = true;
            }
        }

        if window.is_key_pressed(Key::X, minifb::KeyRepeat::No) {
            if let Some((idx, _)) = selected_block {
                objects.remove(idx);
                moved = true;
            }
        }
        let orbit = [
            (Key::Left, ROTATION_SPEED, 0.0),
            (Key::Right, -ROTATION_SPEED, 0.0),
            (Key::Up, 0.0, -ROTATION_SPEED),
            (Key::Down, 0.0, ROTATION_SPEED),
        ];

        for (key, delta_yaw, delta_pitch) in orbit {
            if window.is_key_down(key) {
                camera.orbit(delta_yaw, delta_pitch);
                moved = true;
            }
        }

        // Zoom with W / S keys
        if window.is_key_down(Key::W) {
            camera.zoom(0.96);
            moved = true;
        }
        if window.is_key_down(Key::S) {
            camera.zoom(1.04);
            moved = true;
        }

        // Zoom with mouse scroll wheel
        if let Some((_, scroll_y)) = window.get_scroll_wheel() {
            if scroll_y > 0.0 {
                camera.zoom(0.92);
                moved = true;
            } else if scroll_y < 0.0 {
                camera.zoom(1.08);
                moved = true;
            }
        }

        if moved {
            camera_state = 0;
        }

        if camera_state < 2 {
            let block_size = if camera_state == 0 { 2 } else { 1 }; // Render at half resolution when moving for a balance of speed and quality
            
            let angle = time_of_day * std::f32::consts::PI * 2.0 - std::f32::consts::PI / 2.0;
            let sun_y = angle.sin() * 10.0;
            let sun_x = angle.cos() * 10.0;
            
            let is_day = sun_y > 0.0;
            let intensity = if is_day {
                1.5 * (sun_y / 10.0).clamp(0.2, 1.0)
            } else {
                0.3 // Moonlight
            };

            let light_color = if is_day {
                Color::new(255, 255, 255)
            } else {
                Color::new(100, 100, 255)
            };

            let light_pos = if is_day {
                Vec3::new(sun_x, sun_y, 8.0)
            } else {
                Vec3::new(-sun_x, -sun_y, -8.0)
            };

            let current_light = Light::new(light_pos, light_color, intensity);
            let lava_light = Light::new(Vec3::new(2.0, -3.0, 2.0), Color::new(255, 120, 0), 2.0);

            let selected_index = selected_block.as_ref().map(|(i, _)| *i);

            render(&mut framebuffer, &objects, &camera, &[current_light, lava_light], time_of_day, block_size, selected_index);
            
            draw_ui(&mut framebuffer, &inventory, active_block_index);
            
            // Only advance to high-res if no keys are pressed (we stop moving)
            if !moved {
                camera_state += 1;
            }
        }

        window
            .update_with_buffer(&framebuffer.buffer, WIDTH, HEIGHT)
            .unwrap();

        std::thread::sleep(frame_delay);
    }
}
