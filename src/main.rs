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
mod voxel_grid;

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

use diorama::{build_diorama, build_inventory, build_nether_diorama, tex_mat, DEFAULT_WORLD_SEED};
use render::{render, SkyMode};
use ui::draw_ui;
use ray_intersect::RayIntersect;
use voxel_grid::World;
use pig::Pig;

mod pig;

const WIDTH: usize = 800;
const HEIGHT: usize = 600;
const ROTATION_SPEED: f32 = PI / 60.0;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Realm {
    Overworld,
    Nether,
}

fn main() {
    let frame_delay = Duration::from_millis(16);
    let world_seed = read_world_seed();

    let mut framebuffer = Framebuffer::new(WIDTH, HEIGHT);

    let mut window = Window::new(
        &format!("Minecraft Raytracer Diorama — seed {world_seed}"),
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .unwrap();

    let texture_atlas = Arc::new(Texture::new("assets/textures.png"));
    let pig_texture = Arc::new(Texture::new("assets/pig_temperate.png"));
    let mut overworld = World::from_objects(build_diorama(&texture_atlas, world_seed));
    let mut nether = World::from_objects(build_nether_diorama(&texture_atlas, world_seed ^ 0x4E45_5448_4552));
    
    // Add the Pig to the overworld
    let my_pig = Pig::new(Vec3::new(2.0, 0.5, 2.0), &pig_texture);
    overworld.entities.push(Box::new(my_pig));

    let inventory = build_inventory(&texture_atlas);
    let mut realm = Realm::Overworld;

    let mut active_block_index = 0;

    let mut time_of_day = 0.5; // Noon

    let mut camera = spawn_camera(realm);

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

        let keys = [
            Key::Key1, Key::Key2, Key::Key3, Key::Key4,
            Key::Key5, Key::Key6, Key::Key7,
        ];
        for (i, key) in keys.iter().enumerate() {
            if window.is_key_pressed(*key, minifb::KeyRepeat::No) {
                active_block_index = i;
                moved = true;
            }
        }

        // Block Selection Raycasting
        let center_ray = normalize(&Vec3::new(0.0, 0.0, -1.0));
        let center_ray = camera.basis_change(&center_ray);
        let mut selected_block: Option<([i32; 3], crate::ray_intersect::Intersect)> = None;
        
        {
            let world = match realm {
                Realm::Overworld => &overworld,
                Realm::Nether => &nether,
            };
            
            if let Some(intersect) = world.ray_intersect(&camera.eye, &center_ray) {
                if intersect.distance < 10.0 {
                    let hit_voxel = [
                        (intersect.point.x - intersect.normal.x * 0.01).round() as i32,
                        (intersect.point.y - intersect.normal.y * 0.01).round() as i32,
                        (intersect.point.z - intersect.normal.z * 0.01).round() as i32,
                    ];
                    selected_block = Some((hit_voxel, intersect));
                }
            }
        }

        // La transición sólo se activa al mirar directamente la superficie morada
        // del portal; E sobre la obsidiana no cambia de escena por accidente.
        if window.is_key_pressed(Key::E, minifb::KeyRepeat::No)
            && selected_block
                .as_ref()
                .is_some_and(|(_, hit)| hit.material.is_portal)
        {
            realm = match realm {
                Realm::Overworld => Realm::Nether,
                Realm::Nether => Realm::Overworld,
            };
            // world dynamically selected
            camera = spawn_camera(realm);
            selected_block = None;
            moved = true;
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

                let current_world = match realm {
                    Realm::Overworld => &mut overworld,
                    Realm::Nether => &mut nether,
                };
                let nx = new_center.x.round() as i32;
                let ny = new_center.y.round() as i32;
                let nz = new_center.z.round() as i32;
                current_world.grid.insert(nx, ny, nz, cube);
                moved = true;
            }
        }

        if window.is_key_pressed(Key::X, minifb::KeyRepeat::No) {
            if let Some((voxel, _)) = selected_block {
                let current_world = match realm {
                    Realm::Overworld => &mut overworld,
                    Realm::Nether => &mut nether,
                };
                current_world.grid.remove(voxel[0], voxel[1], voxel[2]);
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

        // Render continuously so animations play!
        let block_size = if camera_state == 0 { 2 } else { 1 }; // Render at half resolution when moving for speed

            let (lights, sky_mode) = match realm {
                Realm::Overworld => {
                    let angle = time_of_day * std::f32::consts::PI * 2.0 - std::f32::consts::PI / 2.0;
                    let sun_y = angle.sin() * 10.0;
                    let sun_x = angle.cos() * 10.0;
                    let is_day = sun_y > 0.0;
                    let intensity = if is_day { 1.5 * (sun_y / 10.0).clamp(0.2, 1.0) } else { 0.3 };
                    let light_color = if is_day { Color::new(255, 255, 255) } else { Color::new(100, 100, 255) };
                    let light_pos = if is_day { Vec3::new(sun_x, sun_y, 8.0) } else { Vec3::new(-sun_x, -sun_y, -8.0) };
                    (
                        vec![
                            Light::new(light_pos, light_color, intensity, 1000.0),
                            // Lava light in mega cave
                            Light::new(Vec3::new(0.0, -22.0, 0.0), Color::new(255, 120, 0), 2.8, 50.0),
                            // Portal lights (front and back) en la nueva isla
                            Light::new(Vec3::new(0.5, 4.5, -64.0), Color::new(200, 50, 255), 2.5, 8.0),
                            Light::new(Vec3::new(0.5, 4.5, -66.0), Color::new(200, 50, 255), 2.5, 8.0),
                        ],
                        SkyMode::Overworld,
                    )
                }
                Realm::Nether => (
                    vec![
                            // Lava light in the center of the Nether lake
                            Light::new(Vec3::new(0.0, 1.0, 0.0), Color::new(255, 70, 12), 3.0, 20.0),
                            // Portal lights at the end of the bridge
                            Light::new(Vec3::new(-0.5, 4.5, 16.0), Color::new(200, 50, 255), 2.5, 8.0),
                            Light::new(Vec3::new(-0.5, 4.5, 14.0), Color::new(200, 50, 255), 2.5, 8.0),
                    ],
                    SkyMode::Nether,
                ),
            };

            let selected_voxel = selected_block.as_ref().map(|(v, _)| *v);

            let current_world = match realm {
                Realm::Overworld => &mut overworld,
                Realm::Nether => &mut nether,
            };

            current_world.update(1.0 / 20.0); // simple fixed timestep

            render(
                &mut framebuffer,
                current_world,
                &camera,
                &lights,
                time_of_day,
                block_size,
                selected_voxel,
                sky_mode,
            );

            draw_ui(&mut framebuffer, &inventory, active_block_index);

            // Only advance to high-res if no keys are pressed (we stop moving)
            if !moved {
                camera_state += 1;
            }

        window
            .update_with_buffer(&framebuffer.buffer, WIDTH, HEIGHT)
            .unwrap();

        std::thread::sleep(frame_delay);
    }
}

fn spawn_camera(realm: Realm) -> Camera {
    match realm {
        Realm::Overworld => Camera::new(
            Vec3::new(0.0, 2.0, 5.5),
            Vec3::new(0.0, 0.3, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
        ),
        Realm::Nether => Camera::new(
            Vec3::new(0.0, 3.0, 11.0),
            Vec3::new(-0.5, 3.2, 4.0),
            Vec3::new(0.0, 1.0, 0.0),
        ),
    }
}

fn read_world_seed() -> u64 {
    let args: Vec<String> = std::env::args().collect();
    args.windows(2)
        .find_map(|pair| (pair[0] == "--seed").then(|| pair[1].parse().ok()).flatten())
        .or_else(|| {
            args.iter().find_map(|arg| {
                arg.strip_prefix("--seed=")
                    .and_then(|value| value.parse().ok())
            })
        })
        .unwrap_or(DEFAULT_WORLD_SEED)
}
