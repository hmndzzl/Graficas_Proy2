use crate::color::Color;
use crate::cube::Cube;
use crate::ray_intersect::{RayIntersect, Material};
use crate::texture::Texture;
use std::sync::Arc;

pub fn tex_mat(atlas: &Arc<Texture>, u: f32, v: f32) -> Material {
    Material::new(Color::new(255, 255, 255), 10.0, [0.8, 0.1, 0.0, 0.0])
        .with_texture(Arc::clone(atlas))
        .with_uv((1.0 / 16.0, 1.0 / 16.0), (u / 16.0, v / 16.0))
}

pub fn build_inventory(texture_atlas: &Arc<Texture>) -> Vec<(&'static str, Material)> {
    let grass_mat = tex_mat(texture_atlas, 3.0, 15.0);
    let planks_mat = tex_mat(texture_atlas, 4.0, 15.0);
    let stone_mat = Material::new(Color::new(255, 255, 255), 15.0, [0.8, 0.1, 0.0, 0.0])
        .with_texture(Arc::clone(texture_atlas))
        .with_uv((1.0 / 16.0, 1.0 / 16.0), (1.0 / 16.0, 15.0 / 16.0));
    let glass_mat = Material::new(Color::new(200, 220, 255), 50.0, [0.1, 0.4, 0.1, 0.8])
        .with_refractive_index(1.5);
    let leaves_mat = Material::new(Color::new(255, 255, 255), 5.0, [0.8, 0.1, 0.0, 0.0])
        .with_texture(Arc::clone(texture_atlas))
        .with_uv((1.0 / 16.0, 1.0 / 16.0), (4.0 / 16.0, 12.0 / 16.0));

    vec![
        ("Césped", grass_mat),
        ("Tablas", planks_mat),
        ("Piedra", stone_mat),
        ("Hojas", leaves_mat),
        ("Cristal", glass_mat),
    ]
}

pub fn build_diorama(texture_atlas: &Arc<Texture>) -> Vec<Box<dyn RayIntersect>> {
    let mut objects: Vec<Box<dyn RayIntersect>> = Vec::new();

    let grass_side = tex_mat(texture_atlas, 3.0, 15.0);
    let grass_top = tex_mat(texture_atlas, 0.0, 15.0);
    
    let stone_mat = Material::new(Color::new(255, 255, 255), 15.0, [0.8, 0.1, 0.0, 0.0])
        .with_texture(Arc::clone(texture_atlas))
        .with_uv((1.0/16.0, 1.0/16.0), (1.0 / 16.0, 15.0 / 16.0));
        
    let gold_mat = Material::new(Color::new(255, 255, 255), 80.0, [0.8, 0.5, 0.2, 0.0])
        .with_texture(Arc::clone(texture_atlas))
        .with_uv((1.0/16.0, 1.0/16.0), (7.0 / 16.0, 14.0 / 16.0));
        
    let diamond_ore_mat = Material::new(Color::new(255, 255, 255), 20.0, [0.8, 0.2, 0.0, 0.0])
        .with_texture(Arc::clone(texture_atlas))
        .with_uv((1.0/16.0, 1.0/16.0), (2.0 / 16.0, 12.0 / 16.0));
    
    let wood_mat = tex_mat(texture_atlas, 4.0, 14.0);
    let wood_top = tex_mat(texture_atlas, 5.0, 14.0);
    let planks_mat = tex_mat(texture_atlas, 4.0, 15.0);
    
    let leaves_mat = Material::new(Color::new(255, 255, 255), 5.0, [0.8, 0.0, 0.0, 0.0])
        .with_texture(Arc::clone(texture_atlas))
        .with_uv((1.0/16.0, 1.0/16.0), (4.0 / 16.0, 12.0 / 16.0));

    let crafting_top = tex_mat(texture_atlas, 11.0, 13.0);
    let crafting_front = tex_mat(texture_atlas, 11.0, 12.0);
    let crafting_side = tex_mat(texture_atlas, 12.0, 12.0);

    let furnace_top = tex_mat(texture_atlas, 14.0, 12.0);
    let furnace_front = tex_mat(texture_atlas, 13.0, 12.0);
    let furnace_side = tex_mat(texture_atlas, 13.0, 13.0);

    let lava_mat = Material::new(Color::new(255, 120, 0), 0.0, [1.0, 0.0, 0.0, 0.0])
        .with_texture(Arc::clone(texture_atlas))
        .with_uv((1.0/16.0, 1.0/16.0), (15.0 / 16.0, 0.0))
        .with_emission(true);
        
    let glass_mat = Material::new(Color::new(200, 220, 255), 50.0, [0.1, 0.4, 0.1, 0.8])
        .with_refractive_index(1.5);
        
    let water_mat = Material::new(Color::new(50, 100, 255), 40.0, [0.2, 0.3, 0.1, 0.6])
        .with_refractive_index(1.33)
        .with_water(true);

    let bed_foot_top = Material::new(Color::new(255, 255, 255), 5.0, [0.8, 0.0, 0.0, 0.0])
        .with_texture(Arc::clone(texture_atlas))
        .with_uv((1.0/16.0, 1.0/16.0), (6.0 / 16.0, 7.0 / 16.0))
        .with_uv_rotated(true);
    let bed_head_top = Material::new(Color::new(255, 255, 255), 5.0, [0.8, 0.0, 0.0, 0.0])
        .with_texture(Arc::clone(texture_atlas))
        .with_uv((1.0/16.0, 1.0/16.0), (7.0 / 16.0, 7.0 / 16.0))
        .with_uv_rotated(true);
        
    let bed_foot_front = tex_mat(texture_atlas, 5.0, 6.0);
    let bed_head_back = tex_mat(texture_atlas, 8.0, 6.0);
    let bed_foot_side = tex_mat(texture_atlas, 6.0, 6.0);
    let bed_head_side = tex_mat(texture_atlas, 7.0, 6.0);

    // Island Base Geometry
    let radius = 6.0;
    for x in -6..=6 {
        for z in -6..=6 {
            for y in -5..=0 {
                let fx = x as f32;
                let fz = z as f32;
                let fy = y as f32;
                
                // Distancia al centro (cilíndrica)
                let dist = (fx * fx + fz * fz).sqrt();

                if dist <= radius {
                    let mut mat = stone_mat.clone();

                    if y == 0 {
                        mat = grass_side.clone();
                    } else if y > -3 {
                        if fast_noise(fx * 0.5, fz * 0.5) > 0.5 {
                            mat = stone_mat.clone();
                        } else {
                            // Dirt isn't declared, we'll reuse grass_side or add it.
                            mat = tex_mat(texture_atlas, 2.0, 15.0);
                        }
                    } else if fast_noise(fx, fz) > 0.8 {
                        mat = diamond_ore_mat.clone();
                    } else if fast_noise(fx * 1.5, fz * 1.5) > 0.7 {
                        mat = gold_mat.clone();
                    }

                    let mut cube = Cube::new(
                        nalgebra_glm::Vec3::new(fx - 0.5, fy - 0.5, fz - 0.5),
                        nalgebra_glm::Vec3::new(fx + 0.5, fy + 0.5, fz + 0.5),
                        mat.clone(),
                    );

                    if y == 0 {
                        cube = cube.with_top_material(grass_top.clone())
                                   .with_bottom_material(tex_mat(texture_atlas, 2.0, 15.0)); // dirt
                    }

                    objects.push(Box::new(cube));
                }
            }
        }
    }

    // Add central tree
    let tree_x = 0.0;
    let tree_z = 0.0;
    let tree_y_start = 1.0;
    let tree_height = 4;

    for y in 0..tree_height {
        let fy = tree_y_start + y as f32;
        let mut cube = Cube::new(
            nalgebra_glm::Vec3::new(tree_x - 0.5, fy - 0.5, tree_z - 0.5),
            nalgebra_glm::Vec3::new(tree_x + 0.5, fy + 0.5, tree_z + 0.5),
            wood_mat.clone(),
        );
        cube = cube.with_top_material(wood_top.clone())
                   .with_bottom_material(wood_top.clone());
        objects.push(Box::new(cube));
    }

    for lx in -2i32..=2 {
        for lz in -2i32..=2 {
            for ly in 0i32..=2 {
                if lx == 0 && lz == 0 && ly < 2 { continue; }
                if (lx.abs() == 2 && lz.abs() == 2) && ly == 2 { continue; }

                let fx = tree_x + lx as f32;
                let fz = tree_z + lz as f32;
                let fy = tree_y_start + tree_height as f32 - 1.0 + ly as f32;

                objects.push(Box::new(Cube::new(
                    nalgebra_glm::Vec3::new(fx - 0.5, fy - 0.5, fz - 0.5),
                    nalgebra_glm::Vec3::new(fx + 0.5, fy + 0.5, fz + 0.5),
                    leaves_mat.clone(),
                )));
            }
        }
    }

    objects
}

fn fast_noise(x: f32, z: f32) -> f32 {
    let seed = x * 12.9898 + z * 78.233;
    (seed.sin() * 43758.5453).fract().abs()
}
