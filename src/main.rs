mod camera;
mod color;
mod cube;
mod diorama;
mod framebuffer;
mod light;
mod ray_intersect;
mod render;
mod sky;
mod texture;
mod ui;

use camera::Camera;
use color::Color;
use cube::Cube;
use framebuffer::Framebuffer;
use light::Light;
use minifb::{Key, Window, WindowOptions};
use nalgebra_glm::{Vec3, normalize};
use std::f32::consts::PI;
use std::sync::Arc;
use std::time::Duration;
use texture::Texture;

use diorama::{build_diorama, build_inventory, tex_mat};
use render::render;
use ui::draw_ui;

const WIDTH: usize = 800;
const HEIGHT: usize = 600;
const ROTATION_SPEED: f32 = PI / 60.0;

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

    let mut objects = build_diorama(&texture_atlas);
    let inventory = build_inventory(&texture_atlas);

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
            if time_of_day >= 1.0 {
                time_of_day -= 1.0;
            }
            moved = true;
        }

        if window.is_key_pressed(Key::Key1, minifb::KeyRepeat::No) {
            active_block_index = 0;
            moved = true;
        }
        if window.is_key_pressed(Key::Key2, minifb::KeyRepeat::No) {
            active_block_index = 1;
            moved = true;
        }
        if window.is_key_pressed(Key::Key3, minifb::KeyRepeat::No) {
            active_block_index = 2;
            moved = true;
        }
        if window.is_key_pressed(Key::Key4, minifb::KeyRepeat::No) {
            active_block_index = 3;
            moved = true;
        }
        if window.is_key_pressed(Key::Key5, minifb::KeyRepeat::No) {
            active_block_index = 4;
            moved = true;
        }

        // Block Selection Raycasting
        let center_ray = normalize(&Vec3::new(0.0, 0.0, -1.0));
        let center_ray = camera.basis_change(&center_ray);
        let mut selected_block: Option<(usize, crate::ray_intersect::Intersect)> = None;
        for (i, object) in objects.iter().enumerate() {
            if let Some(intersect) = object.ray_intersect(&camera.eye, &center_ray) {
                if intersect.distance < 10.0 {
                    if selected_block
                        .as_ref()
                        .is_none_or(|(_, current)| intersect.distance < current.distance)
                    {
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
                let new_center =
                    Vec3::new(hit_center_x, hit_center_y, hit_center_z) + intersect.normal;

                let active_name = inventory[active_block_index].0;
                let active_mat = inventory[active_block_index].1.clone();
                let mut cube = Cube::new(
                    new_center - Vec3::new(0.5, 0.5, 0.5),
                    new_center + Vec3::new(0.5, 0.5, 0.5),
                    active_mat,
                );

                if active_name == "Césped" {
                    cube = cube
                        .with_top_material(tex_mat(&texture_atlas, 0.0, 15.0))
                        .with_bottom_material(tex_mat(&texture_atlas, 2.0, 15.0)); // dirt
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

            render(
                &mut framebuffer,
                &objects,
                &camera,
                &[current_light, lava_light],
                time_of_day,
                block_size,
                selected_index,
            );

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
