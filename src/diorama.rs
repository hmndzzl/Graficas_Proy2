use crate::color::Color;
use crate::cube::Cube;
use crate::ray_intersect::{Material, RayIntersect};
use crate::texture::Texture;
use nalgebra_glm::Vec3;
use std::sync::Arc;

/// Cambiar este valor crea otra variante reproducible de la isla.
pub const DEFAULT_WORLD_SEED: u64 = 20260930;

pub fn tex_mat(atlas: &Arc<Texture>, u: f32, v: f32) -> Material {
    Material::new(Color::new(255, 255, 255), 10.0, [0.8, 0.1, 0.0, 0.0])
        .with_texture(Arc::clone(atlas))
        .with_uv((1.0 / 16.0, 1.0 / 16.0), (u / 16.0, v / 16.0))
}

/// Convierte una celda indicada visualmente desde arriba a la convención ya
/// existente de `tex_mat`, cuyo eje Y está invertido por `get_color`.
fn tex_mat_from_top(atlas: &Arc<Texture>, u: f32, top_v: f32) -> Material {
    tex_mat(atlas, u, 15.0 - top_v)
}

fn obsidian_mat(atlas: &Arc<Texture>) -> Material {
    // Coordenada indicada en el atlas: obsidiana (5, 2).
    tex_mat_from_top(atlas, 5.0, 2.0)
}

fn portal_mat(atlas: &Arc<Texture>) -> Material {
    // Coordenada indicada en el atlas: interior del portal (14, 0).
    tex_mat_from_top(atlas, 14.0, 0.0)
        .with_emission(true)
        .with_portal(true)
}

pub fn build_inventory(texture_atlas: &Arc<Texture>) -> Vec<(&'static str, Material)> {
    let grass_mat = tex_mat(texture_atlas, 3.0, 15.0);
    let planks_mat = tex_mat(texture_atlas, 4.0, 15.0);
    let stone_mat = tex_mat(texture_atlas, 1.0, 15.0);
    let glass_mat = Material::new(Color::new(200, 220, 255), 50.0, [0.1, 0.4, 0.1, 0.8])
        .with_refractive_index(1.5);
    let leaves_mat = tex_mat(texture_atlas, 4.0, 12.0);

    vec![
        ("Césped", grass_mat),
        ("Tablas", planks_mat),
        ("Piedra", stone_mat),
        ("Hojas", leaves_mat),
        ("Cristal", glass_mat),
        ("Obsidiana", obsidian_mat(texture_atlas)),
        ("Portal", portal_mat(texture_atlas)),
    ]
}

/// Construye una isla inicial amplia y reproducible.
/// La semilla cambia el contorno, variaciones del terreno, árboles y vetas,
/// mientras que mantiene los puntos importantes para que se puedan recorrer.
pub fn build_diorama(texture_atlas: &Arc<Texture>, seed: u64) -> Vec<Box<dyn RayIntersect>> {
    let mut objects: Vec<Box<dyn RayIntersect>> = Vec::new();

    let grass_side = tex_mat(texture_atlas, 3.0, 15.0);
    let grass_top = tex_mat(texture_atlas, 0.0, 15.0);
    let dirt_mat = tex_mat(texture_atlas, 2.0, 15.0);
    let stone_mat = tex_mat(texture_atlas, 1.0, 15.0);
    let gold_mat = Material::new(Color::new(255, 255, 255), 80.0, [0.8, 0.5, 0.2, 0.0])
        .with_texture(Arc::clone(texture_atlas))
        .with_uv((1.0 / 16.0, 1.0 / 16.0), (7.0 / 16.0, 14.0 / 16.0));
    let diamond_ore_mat = Material::new(Color::new(255, 255, 255), 20.0, [0.8, 0.2, 0.0, 0.0])
        .with_texture(Arc::clone(texture_atlas))
        .with_uv((1.0 / 16.0, 1.0 / 16.0), (2.0 / 16.0, 12.0 / 16.0));
    let wood_mat = tex_mat(texture_atlas, 4.0, 14.0);
    let wood_top = tex_mat(texture_atlas, 5.0, 14.0);
    let planks_mat = tex_mat(texture_atlas, 4.0, 15.0);
    let leaves_mat = tex_mat(texture_atlas, 4.0, 12.0);
    let glass_mat = Material::new(Color::new(200, 220, 255), 50.0, [0.1, 0.4, 0.1, 0.65])
        .with_refractive_index(1.5);
    let water_mat = Material::new(Color::new(50, 115, 255), 40.0, [0.15, 0.3, 0.08, 0.58])
        .with_refractive_index(1.33)
        .with_water(true);
    let lava_mat = tex_mat(texture_atlas, 15.0, 0.0).with_emission(true);

    // Superficie amplia. El lago queda excavado para que el agua no flote.
    const RADIUS: i32 = 16;
    for x in -RADIUS..=RADIUS {
        for z in -RADIUS..=RADIUS {
            let distance = ((x * x + z * z) as f32).sqrt();
            let edge = seeded_noise(seed, x, z) * 0.85;
            if distance > RADIUS as f32 - 0.35 + edge || is_lake(x, z) {
                continue;
            }

            let block = unit_cube(x as f32, 0.0, z as f32, grass_side.clone())
                .with_top_material(grass_top.clone())
                .with_bottom_material(dirt_mat.clone());
            objects.push(Box::new(block));
        }
    }

    // Volumen de la isla: cada posición se instancia como un bloque individual.
    // La forma se estrecha hacia abajo y deja libre la abertura de la cueva.
    add_voxel_island(&mut objects, RADIUS, &dirt_mat, &stone_mat, seed);

    add_lake(&mut objects, &water_mat);
    add_cave(&mut objects, &stone_mat, &diamond_ore_mat, &gold_mat, &lava_mat, seed);
    add_cabin(
        &mut objects,
        texture_atlas,
        &planks_mat,
        &wood_mat,
        &wood_top,
        &glass_mat,
    );

    // El portal mira hacia el frente (+Z), por lo que se reconoce al iniciar.
    add_portal(
        &mut objects,
        6,
        4,
        &obsidian_mat(texture_atlas),
        &portal_mat(texture_atlas),
    );

    add_tree(&mut objects, -1, -6, 4, &wood_mat, &wood_top, &leaves_mat);
    add_tree(&mut objects, 8, -5, 5, &wood_mat, &wood_top, &leaves_mat);
    add_tree(&mut objects, -10, 4, 4 + (seeded_noise(seed, -10, 4) > 0.5) as i32, &wood_mat, &wood_top, &leaves_mat);
    add_tree(&mut objects, 1, 9, 4 + (seeded_noise(seed, 1, 9) > 0.55) as i32, &wood_mat, &wood_top, &leaves_mat);

    objects
}

/// Segundo mundo: un Bosque Carmesí del Nether independiente de la isla.
/// Comparte la seed para que sus variaciones también sean reproducibles.
pub fn build_nether_diorama(texture_atlas: &Arc<Texture>, seed: u64) -> Vec<Box<dyn RayIntersect>> {
    let mut objects: Vec<Box<dyn RayIntersect>> = Vec::new();
    // Todas las superficies del Nether vienen del atlas existente.
    let netherrack = tex_mat_from_top(texture_atlas, 7.0, 6.0);
    let soul_sand = tex_mat_from_top(texture_atlas, 8.0, 6.0);
    let glowstone = tex_mat_from_top(texture_atlas, 9.0, 6.0).with_emission(true);
    let lava = tex_mat(texture_atlas, 15.0, 0.0).with_emission(true);

    const RADIUS: i32 = 12;
    for x in -RADIUS..=RADIUS {
        for z in -RADIUS..=RADIUS {
            let distance = ((x * x + z * z) as f32).sqrt();
            if distance > RADIUS as f32 - 0.25 + seeded_noise(seed ^ 0x4E45_5448_4552, x, z) * 0.8 {
                continue;
            }
            let material = if z < -4 || x < -7 {
                soul_sand.clone()
            } else {
                netherrack.clone()
            };
            objects.push(Box::new(unit_cube(x as f32, 0.0, z as f32, material)));
        }
    }

    // Núcleo flotante y desniveles: sólo netherrack y soul sand del atlas.
    add_box(&mut objects, Vec3::new(-10.5, -3.0, -10.5), Vec3::new(10.5, -0.5, 10.5), netherrack.clone());
    add_box(&mut objects, Vec3::new(-7.5, -5.5, -7.5), Vec3::new(7.5, -3.0, 7.5), soul_sand.clone());
    add_box(&mut objects, Vec3::new(-4.5, -7.0, -4.5), Vec3::new(4.5, -5.5, 4.5), netherrack.clone());

    for &(x, z, height) in &[(-9, -2, 5), (-7, 5, 3), (7, 5, 5), (9, -1, 4), (3, -8, 3)] {
        for y in 1..=height {
            let material = if y == height { soul_sand.clone() } else { netherrack.clone() };
            objects.push(Box::new(unit_cube(x as f32, y as f32, z as f32, material)));
        }
    }

    // Lagos y cascadas de lava que iluminan el nuevo bioma.
    for x in 3..=7 {
        for z in -2..=2 {
            add_box(&mut objects, Vec3::new(x as f32 - 0.5, -0.48, z as f32 - 0.5), Vec3::new(x as f32 + 0.5, 0.1, z as f32 + 0.5), lava.clone());
        }
    }
    for y in 1..=5 {
        objects.push(Box::new(unit_cube(8.0, y as f32, -3.0, lava.clone())));
    }

    add_nether_fungus(&mut objects, -6, -5, 5, &netherrack, &soul_sand);
    add_nether_fungus(&mut objects, -3, -8, 4, &netherrack, &soul_sand);
    add_nether_fungus(&mut objects, 2, -7, 5, &netherrack, &soul_sand);
    add_nether_fungus(&mut objects, -9, 3, 4, &netherrack, &soul_sand);

    // Racimos de glowstone para contraste y una luz cálida superior.
    for &(x, y, z) in &[(-3, 7, -1), (-2, 7, -1), (-3, 6, -1), (4, 8, -5), (4, 7, -5)] {
        objects.push(Box::new(unit_cube(x as f32, y as f32, z as f32, glowstone.clone())));
    }

    // Portal de regreso, orientado al frente para que sea visible al aparecer.
    add_portal(
        &mut objects,
        -2,
        4,
        &obsidian_mat(texture_atlas),
        &portal_mat(texture_atlas),
    );
    objects
}

fn is_lake(x: i32, z: i32) -> bool {
    (3..=7).contains(&x) && (-5..=-1).contains(&z) && !((x == 3 || x == 7) && (z == -5 || z == -1))
}

fn unit_cube(x: f32, y: f32, z: f32, material: Material) -> Cube {
    Cube::new(Vec3::new(x - 0.5, y - 0.5, z - 0.5), Vec3::new(x + 0.5, y + 0.5, z + 0.5), material)
}

fn add_voxel_island(
    objects: &mut Vec<Box<dyn RayIntersect>>,
    surface_radius: i32,
    dirt: &Material,
    stone: &Material,
    seed: u64,
) {
    for y in -7..=-1 {
        let radius = match y {
            -1 => surface_radius - 1,
            -2 => surface_radius - 2,
            -3 => surface_radius - 3,
            -4 => surface_radius - 4,
            -5 => surface_radius - 6,
            -6 => surface_radius - 8,
            _ => surface_radius - 10,
        };

        for x in -radius..=radius {
            for z in -radius..=radius {
                let distance = ((x * x + z * z) as f32).sqrt();
                let edge_noise = seeded_noise(seed ^ 0x1A1A_0001, x, z) * 0.45;
                if distance > radius as f32 + edge_noise || is_cave_void(x, y, z) {
                    continue;
                }
                let material = if y >= -2 { dirt.clone() } else { stone.clone() };
                objects.push(Box::new(unit_cube(x as f32, y as f32, z as f32, material)));
            }
        }
    }
}

fn is_cave_void(x: i32, y: i32, z: i32) -> bool {
    (-4..=4).contains(&x) && (-4..=-1).contains(&y) && (-2..=16).contains(&z)
}

fn add_box(objects: &mut Vec<Box<dyn RayIntersect>>, min: Vec3, max: Vec3, material: Material) {
    let first_x = (min.x + 0.5).round() as i32;
    let first_y = (min.y + 0.5).round() as i32;
    let first_z = (min.z + 0.5).round() as i32;
    let last_x = (max.x - 0.5).round() as i32;
    let last_y = (max.y - 0.5).round() as i32;
    let last_z = (max.z - 0.5).round() as i32;

    for x in first_x..=last_x {
        for y in first_y..=last_y {
            for z in first_z..=last_z {
                objects.push(Box::new(unit_cube(x as f32, y as f32, z as f32, material.clone())));
            }
        }
    }
}

fn add_lake(objects: &mut Vec<Box<dyn RayIntersect>>, water_mat: &Material) {
    for x in 3..=7 {
        for z in -5..=-1 {
            if is_lake(x, z) {
                add_box(objects, Vec3::new(x as f32 - 0.5, -0.48, z as f32 - 0.5), Vec3::new(x as f32 + 0.5, 0.12, z as f32 + 0.5), water_mat.clone());
            }
        }
    }
}

fn add_cave(objects: &mut Vec<Box<dyn RayIntersect>>, stone: &Material, diamond: &Material, gold: &Material, lava: &Material, seed: u64) {
    // Pared del fondo y laterales de una cueva abierta hacia +Z.
    for y in -4..=0 {
        for x in -4..=4 {
            let material = if y < -1 && seeded_noise(seed, x, y) > 0.72 {
                diamond.clone()
            } else if y < -2 && seeded_noise(seed ^ 0x9E37_79B9, x, y) > 0.82 {
                gold.clone()
            } else {
                stone.clone()
            };
            objects.push(Box::new(unit_cube(x as f32, y as f32, -2.0, material)));
        }
    }

    for z in -1..=5 {
        for y in -4..=0 {
            objects.push(Box::new(unit_cube(-4.0, y as f32, z as f32, stone.clone())));
            objects.push(Box::new(unit_cube(4.0, y as f32, z as f32, stone.clone())));
        }
    }

    // Vetas garantizadas y visibles desde la entrada, independientemente de la seed.
    for (x, y) in [(-2, -2), (-1, -3), (1, -2), (2, -3)] {
        objects.push(Box::new(unit_cube(x as f32, y as f32, -1.48, diamond.clone())));
    }
    objects.push(Box::new(unit_cube(3.0, -3.0, -1.48, gold.clone())));

    for x in -2..=2 {
        for z in 0..=3 {
            objects.push(Box::new(unit_cube(x as f32, -4.0, z as f32, lava.clone())));
        }
    }
}

fn add_cabin(
    objects: &mut Vec<Box<dyn RayIntersect>>,
    texture_atlas: &Arc<Texture>,
    planks: &Material,
    wood: &Material,
    wood_top: &Material,
    glass: &Material,
) {
    // Cabaña de 7x7 al oeste del lago: suelo, estructura, ventanas y techo.
    for x in -10..=-4 {
        for z in -4..=2 {
            objects.push(Box::new(unit_cube(x as f32, 1.0, z as f32, planks.clone())));
        }
    }

    for &(x, z) in &[(-10, -4), (-10, 2), (-4, -4), (-4, 2)] {
        for y in 2..=5 {
            let pillar = unit_cube(x as f32, y as f32, z as f32, wood.clone())
                .with_top_material(wood_top.clone())
                .with_bottom_material(wood_top.clone());
            objects.push(Box::new(pillar));
        }
    }

    for y in 2..=4 {
        for x in -9..=-5 {
            if x == -7 && y <= 3 { continue; } // puerta en +Z
            objects.push(Box::new(unit_cube(x as f32, y as f32, 2.0, planks.clone())));
            objects.push(Box::new(unit_cube(x as f32, y as f32, -4.0, planks.clone())));
        }
        for z in -3..=1 {
            let side_material = if y == 3 && (z == -2 || z == 0) { glass.clone() } else { planks.clone() };
            objects.push(Box::new(unit_cube(-10.0, y as f32, z as f32, side_material.clone())));
            objects.push(Box::new(unit_cube(-4.0, y as f32, z as f32, side_material)));
        }
    }

    // Techo escalonado, construido igual que el resto: un bloque por posición.
    for x in -11..=-3 {
        for z in -5..=3 {
            if x == -11 || x == -3 || z == -5 || z == 3 {
                objects.push(Box::new(unit_cube(x as f32, 5.0, z as f32, planks.clone())));
            }
        }
    }
    for x in -10..=-4 {
        for z in -4..=2 {
            objects.push(Box::new(unit_cube(x as f32, 6.0, z as f32, planks.clone())));
        }
    }

    // Porche frente a la entrada, cumbrera y chimenea: también son voxeles.
    for x in -8..=-6 {
        objects.push(Box::new(unit_cube(x as f32, 1.0, 3.0, planks.clone())));
        objects.push(Box::new(unit_cube(x as f32, 7.0, -1.0, wood.clone())));
    }
    let chimney_stone = tex_mat(texture_atlas, 1.0, 15.0);
    for y in 6..=8 {
        objects.push(Box::new(unit_cube(-9.0, y as f32, -3.0, chimney_stone.clone())));
    }

    // Interior: se reactivan los bloques ya disponibles en el atlas.
    let crafting_top = tex_mat(texture_atlas, 11.0, 13.0);
    let crafting_front = tex_mat(texture_atlas, 11.0, 12.0);
    let crafting_side = tex_mat(texture_atlas, 12.0, 12.0);
    let furnace_top = tex_mat(texture_atlas, 14.0, 12.0);
    let furnace_front = tex_mat(texture_atlas, 13.0, 12.0);
    let furnace_side = tex_mat(texture_atlas, 13.0, 13.0);
    let bed_foot_top = tex_mat(texture_atlas, 6.0, 7.0).with_uv_rotated(true);
    let bed_head_top = tex_mat(texture_atlas, 7.0, 7.0).with_uv_rotated(true);
    let bed_foot_front = tex_mat(texture_atlas, 5.0, 6.0);
    let bed_head_back = tex_mat(texture_atlas, 8.0, 6.0);
    let bed_foot_side = tex_mat(texture_atlas, 6.0, 6.0);
    let bed_head_side = tex_mat(texture_atlas, 7.0, 6.0);

    let crafting_table = unit_cube(-9.0, 2.0, -3.0, crafting_side.clone())
        .with_top_material(crafting_top)
        .with_front_material(crafting_front.clone())
        .with_back_material(crafting_front)
        .with_left_material(crafting_side.clone())
        .with_right_material(crafting_side);
    objects.push(Box::new(crafting_table));

    let furnace = unit_cube(-5.0, 2.0, -3.0, furnace_side.clone())
        .with_top_material(furnace_top)
        .with_front_material(furnace_front.clone())
        .with_back_material(furnace_front)
        .with_left_material(furnace_side.clone())
        .with_right_material(furnace_side);
    objects.push(Box::new(furnace));

    let bed_foot = unit_cube(-7.0, 2.0, 0.0, bed_foot_side.clone())
        .with_top_material(bed_foot_top)
        .with_front_material(bed_foot_front.clone())
        .with_back_material(bed_foot_front)
        .with_left_material(bed_foot_side.clone())
        .with_right_material(bed_foot_side);
    let bed_head = unit_cube(-7.0, 2.0, -1.0, bed_head_side.clone())
        .with_top_material(bed_head_top)
        .with_back_material(bed_head_back)
        .with_left_material(bed_head_side.clone())
        .with_right_material(bed_head_side);
    objects.push(Box::new(bed_foot));
    objects.push(Box::new(bed_head));
}

fn add_portal(objects: &mut Vec<Box<dyn RayIntersect>>, origin_x: i32, origin_z: i32, obsidian: &Material, portal: &Material) {
    // Marco exterior 4x6; interior morado 2x4.
    for x in origin_x..=origin_x + 3 {
        objects.push(Box::new(unit_cube(x as f32, 1.0, origin_z as f32, obsidian.clone())));
        objects.push(Box::new(unit_cube(x as f32, 6.0, origin_z as f32, obsidian.clone())));
    }
    for y in 2..=5 {
        objects.push(Box::new(unit_cube(origin_x as f32, y as f32, origin_z as f32, obsidian.clone())));
        objects.push(Box::new(unit_cube((origin_x + 3) as f32, y as f32, origin_z as f32, obsidian.clone())));
        for x in origin_x + 1..=origin_x + 2 {
            objects.push(Box::new(unit_cube(x as f32, y as f32, origin_z as f32, portal.clone())));
        }
    }
}

fn add_tree(objects: &mut Vec<Box<dyn RayIntersect>>, x: i32, z: i32, height: i32, wood: &Material, wood_top: &Material, leaves: &Material) {
    for y in 1..=height {
        let trunk = unit_cube(x as f32, y as f32, z as f32, wood.clone())
            .with_top_material(wood_top.clone())
            .with_bottom_material(wood_top.clone());
        objects.push(Box::new(trunk));
    }

    for dx in -2i32..=2 {
        for dz in -2i32..=2 {
            for dy in -1i32..=1 {
                if dx.abs() + dz.abs() + dy.abs() > 4 || (dx == 0 && dz == 0 && dy < 1) { continue; }
                objects.push(Box::new(unit_cube((x + dx) as f32, (height + dy + 1) as f32, (z + dz) as f32, leaves.clone())));
            }
        }
    }
}

fn add_nether_fungus(
    objects: &mut Vec<Box<dyn RayIntersect>>,
    x: i32,
    z: i32,
    height: i32,
    stem: &Material,
    wart: &Material,
) {
    for y in 1..=height {
        objects.push(Box::new(unit_cube(x as f32, y as f32, z as f32, stem.clone())));
    }
    for dx in -2i32..=2 {
        for dz in -2i32..=2 {
            if dx.abs() + dz.abs() > 3 || (dx == 0 && dz == 0) {
                continue;
            }
            objects.push(Box::new(unit_cube((x + dx) as f32, (height + 1) as f32, (z + dz) as f32, wart.clone())));
            if dx.abs() + dz.abs() <= 2 {
                objects.push(Box::new(unit_cube((x + dx) as f32, (height + 2) as f32, (z + dz) as f32, wart.clone())));
            }
        }
    }
    objects.push(Box::new(unit_cube(x as f32, (height + 2) as f32, z as f32, wart.clone())));
}

fn seeded_noise(seed: u64, x: i32, z: i32) -> f32 {
    let mut value = seed
        ^ (x as i64 as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)
        ^ (z as i64 as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F);
    value ^= value >> 30;
    value = value.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    value ^= value >> 27;
    value = value.wrapping_mul(0x94D0_49BB_1331_11EB);
    value ^= value >> 31;
    (value as f64 / u64::MAX as f64) as f32
}
