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
        ("Ladrillos", tex_mat_from_top(texture_atlas, 6.0, 3.0)),
        ("L. Musgoso", tex_mat_from_top(texture_atlas, 4.0, 6.0)),
        ("L. Roto", tex_mat_from_top(texture_atlas, 5.0, 6.0)),
        ("Obsidiana", obsidian_mat(texture_atlas)),
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
    let lamp_mat = Material::new(Color::new(255, 250, 220), 0.0, [1.0, 0.0, 0.0, 0.0]).with_emission(true);
    
    // Enchanting Table Materials
    let ench_top = tex_mat_from_top(texture_atlas, 6.0, 10.0);
    let ench_bot = tex_mat_from_top(texture_atlas, 7.0, 10.0);
    let ench_side1 = tex_mat_from_top(texture_atlas, 6.0, 11.0);
    let ench_side2 = tex_mat_from_top(texture_atlas, 7.0, 11.0);
    
    // Bookshelves (standard Minecraft pos: side 3,2, top/bottom wood planks 4,0)
    let bookshelf_mat = tex_mat_from_top(texture_atlas, 3.0, 2.0);
    
    // End Portal Materials
    let end_stone_mat = tex_mat_from_top(texture_atlas, 15.0, 10.0);
    let end_frame_top = tex_mat_from_top(texture_atlas, 14.0, 9.0);
    let end_frame_bot = tex_mat_from_top(texture_atlas, 15.0, 10.0);
    // Custom UV for side to avoid the top 3 transparent pixels (white stripe)
    let end_frame_side = tex_mat_from_top(texture_atlas, 15.0, 9.0)
        .with_uv((1.0 / 16.0, 13.0 / 256.0), (15.0 / 16.0, 6.0 / 16.0));

    // End Portal Inside Mat (Pitch black space with emission)
    let end_portal_inside = tex_mat_from_top(texture_atlas, 15.0, 12.0)
        .with_emission(true)
        .with_portal(true);

    const RADIUS: i32 = 36;
    
    // --- Isla Principal ---
    add_island_surface(&mut objects, 0, 0, RADIUS, false, &grass_side, &grass_top, &dirt_mat, seed);
    add_voxel_island(&mut objects, 0, 0, RADIUS, false, &dirt_mat, &stone_mat, seed);
    
    // --- Isla Pequeña (Portal en -Z) ---
    const SMALL_RADIUS: i32 = 12;
    add_island_surface(&mut objects, 0, -65, SMALL_RADIUS, true, &grass_side, &grass_top, &dirt_mat, seed ^ 0x9999);
    add_voxel_island(&mut objects, 0, -65, SMALL_RADIUS, true, &dirt_mat, &stone_mat, seed ^ 0x9999);
    
    // --- Isla Pequeña (Encantamientos verdaderamente separada) ---
    add_island_surface(&mut objects, 10, 55, SMALL_RADIUS, true, &grass_side, &grass_top, &dirt_mat, seed ^ 0x8888);
    add_voxel_island(&mut objects, 10, 55, SMALL_RADIUS, true, &dirt_mat, &stone_mat, seed ^ 0x8888);
    
    // --- Stronghold Portal Room (End Portal en -X) ---
    let stone_brick = tex_mat_from_top(texture_atlas, 6.0, 3.0);
    let mossy_brick = tex_mat_from_top(texture_atlas, 4.0, 6.0);
    let cracked_brick = tex_mat_from_top(texture_atlas, 5.0, 6.0);
    
    // Generar el cuarto de stronghold (base de 11x11)
    for sx in -5i32..=5 {
        for sz in -5i32..=5 {
            let px = -55 + sx;
            let pz = 0 + sz;
            
            // Elegir aleatoriamente el tipo de ladrillo
            let rand_val = seeded_noise(seed, px, pz);
            let brick_mat = if rand_val < 0.2 {
                mossy_brick.clone()
            } else if rand_val < 0.4 {
                cracked_brick.clone()
            } else {
                stone_brick.clone()
            };
            
            // Piso base
            // No ponemos piso en el 3x3 central (sx de -1 a 1, sz de -1 a 1) para la lava
            if sx > -2 && sx < 2 && sz > -2 && sz < 2 {
                for dy in 1..=3 {
                    objects.push(Box::new(unit_cube(px as f32, -dy as f32, pz as f32, brick_mat.clone())));
                }
            } else {
                for dy in 0..=3 {
                    objects.push(Box::new(unit_cube(px as f32, -dy as f32, pz as f32, brick_mat.clone())));
                }
            }
            
            // Paredes (altura 1 a 4)
            if sx.abs() == 5 || sz.abs() == 5 {
                // Dejar puerta en sx = 5 (hacia el puente colgante)
                if sx == 5 && sz.abs() <= 1 {
                    continue; 
                }
                for dy in 1..=4 {
                    objects.push(Box::new(unit_cube(px as f32, dy as f32, pz as f32, brick_mat.clone())));
                }
            }
        }
    }
    
    // Escaleras y Spawner al frente (sx = 3)
    let spawner_mat = tex_mat(texture_atlas, 1.0, 4.0); // textura de mob spawner
    objects.push(Box::new(Cube::new(Vec3::new(-52.5, 0.5, -1.5), Vec3::new(-51.5, 1.0, -0.5), stone_brick.clone())));
    objects.push(Box::new(Cube::new(Vec3::new(-52.5, 0.5, 0.5), Vec3::new(-51.5, 1.0, 1.5), stone_brick.clone())));
    objects.push(Box::new(Cube::new(Vec3::new(-52.5, 0.5, -0.5), Vec3::new(-51.5, 1.0, 0.5), spawner_mat.clone())));
    
    // --- Puente Colgante Decorado ---
    for z in -53..=-35 {
        for x in -2..=2 {
            // Un poco de hundimiento en el centro del puente para darle efecto colgante
            let drop = -((z + 44) as f32 / 9.0).powi(2) * 0.5 + 0.5; // arco suave
            let y_bridge = -drop;
            
            if x >= -1 && x <= 1 {
                // Suelo del puente
                objects.push(Box::new(unit_cube(x as f32, y_bridge, z as f32, planks_mat.clone())));
            } else {
                // Postes y barandales
                if z % 3 == 0 {
                    objects.push(Box::new(unit_cube(x as f32, y_bridge + 1.0, z as f32, wood_mat.clone())));
                }
                objects.push(Box::new(unit_cube(x as f32, y_bridge + 0.5, z as f32, planks_mat.clone())));
                
                // Pilares de soporte profundos cada 6 bloques
                if z % 6 == 0 {
                    for y in -25..=(y_bridge as i32) {
                        objects.push(Box::new(unit_cube(x as f32, y as f32, z as f32, wood_mat.clone())));
                    }
                }
            }
        }
    }

    add_lake(&mut objects, &water_mat);
    add_cave(&mut objects, &stone_mat, &diamond_ore_mat, &lava_mat, seed);
    let _white_concrete = Material::new(Color::new(245, 245, 250), 5.0, [0.8, 0.2, 0.0, 0.0]);
    let _pool_water = Material::new(Color::new(100, 180, 255), 100.0, [0.1, 0.4, 0.1, 0.8]).with_refractive_index(1.33);

    add_lodge(
        &mut objects,
        texture_atlas,
        &stone_mat,
        &planks_mat,
        &wood_mat,
        &wood_top,
        &glass_mat,
        &lamp_mat,
    );

    // Plataforma (Altar) para el portal
    let netherrack = tex_mat_from_top(texture_atlas, 7.0, 6.0);
    let glowstone = tex_mat_from_top(texture_atlas, 9.0, 6.0).with_emission(true);
    add_portal_structure(&mut objects, -1, -65, &netherrack, &glowstone);

    // El portal mira hacia el frente (+Z), sobre la plataforma
    add_portal(
        &mut objects,
        -1, // origin_x
        2,  // origin_y
        -65, // origin_z
        &obsidian_mat(texture_atlas),
        &portal_mat(texture_atlas),
    );
    
    // --- Puente Hacia los Encantamientos (cruzando la cueva en z=20) ---
    for z in 15..=50 {
        for x in 8..=12 {
            let drop = -((z - 32) as f32 / 17.0).powi(2) * 1.5 + 1.5;
            let y_bridge = 1.0 - drop;
            
            if x >= 9 && x <= 11 {
                objects.push(Box::new(unit_cube(x as f32, y_bridge, z as f32, planks_mat.clone())));
            } else {
                if z % 3 == 0 {
                    objects.push(Box::new(unit_cube(x as f32, y_bridge + 1.0, z as f32, wood_mat.clone())));
                }
                objects.push(Box::new(unit_cube(x as f32, y_bridge + 0.5, z as f32, planks_mat.clone())));
                
                if z % 6 == 0 {
                    for step in 1..=4 {
                        objects.push(Box::new(unit_cube(x as f32, y_bridge - step as f32, z as f32, wood_mat.clone())));
                    }
                }
            }
        }
    }
    
    // --- Zona de Encantamientos (Altar de Madera) ---
    // Plataforma de madera para el altar en y=1
    for bx in 8..=12 {
        for bz in 53..=57 {
            if bx == 8 || bx == 12 || bz == 53 || bz == 57 {
                objects.push(Box::new(unit_cube(bx as f32, 1.0, bz as f32, wood_mat.clone())));
            } else {
                objects.push(Box::new(unit_cube(bx as f32, 1.0, bz as f32, planks_mat.clone())));
            }
        }
    }
    
    // Mesa de encantamientos en el centro de la isla, elevada sobre el altar (10, 2, 55)
    let ench_cube = unit_cube(10.0, 2.0, 55.0, ench_side1)
        .with_top_material(ench_top)
        .with_bottom_material(ench_bot)
        .with_left_material(ench_side2.clone())
        .with_right_material(ench_side2);
    objects.push(Box::new(ench_cube));
    
    // Librerías alrededor de la mesa
    // Un círculo con 1 bloque de aire entre la mesa y las librerías
    for bx in 8..=12 {
        for bz in 53..=57 {
            // Colocar 2 pisos de librerías en el perímetro exterior (y=2, y=3)
            if bx == 8 || bx == 12 || bz == 53 || bz == 57 {
                // Entrada del lado del puente (z=53, x=9..11)
                if bz == 53 && bx >= 9 && bx <= 11 { continue; }
                
                let bs_cube_1 = unit_cube(bx as f32, 2.0, bz as f32, bookshelf_mat.clone())
                    .with_top_material(planks_mat.clone())
                    .with_bottom_material(planks_mat.clone());
                objects.push(Box::new(bs_cube_1));
                
                let bs_cube_2 = unit_cube(bx as f32, 3.0, bz as f32, bookshelf_mat.clone())
                    .with_top_material(planks_mat.clone())
                    .with_bottom_material(planks_mat.clone());
                objects.push(Box::new(bs_cube_2));
            }
        }
    }
    // (Glowstone eliminada para dejarlo al descubierto)
    
    // --- Puente hacia el End Portal (en -X) ---
    for x in -43..=-35 {
        for z in -2..=2 {
            let drop = -((x + 39) as f32 / 5.0).powi(2) * 0.3 + 0.5;
            let y_bridge = 1.0 - drop;
            
            if z >= -1 && z <= 1 {
                objects.push(Box::new(unit_cube(x as f32, y_bridge, z as f32, planks_mat.clone())));
            } else {
                if x % 2 == 0 {
                    objects.push(Box::new(unit_cube(x as f32, y_bridge + 1.0, z as f32, wood_mat.clone())));
                }
                objects.push(Box::new(unit_cube(x as f32, y_bridge + 0.5, z as f32, planks_mat.clone())));
                
                if x % 4 == 0 {
                    for step in 1..=4 {
                        objects.push(Box::new(unit_cube(x as f32, y_bridge - step as f32, z as f32, wood_mat.clone())));
                    }
                }
            }
        }
    }
    
    // --- Zona del End Portal ---
    let portal_cx = -55;
    let portal_cz = 0;
    
    // Crear el anillo del portal de 3x3 (5x5 exterior con esquinas vacías)
    for px in -2..=2 {
        for pz in -2..=2 {
            // Saltamos el interior (3x3 interior)
            if px > -2 && px < 2 && pz > -2 && pz < 2 {
                // Rellenamos el interior con el material del portal del End
                // El portal del End es más delgado, lo ponemos un poco debajo del tope del frame
                objects.push(Box::new(Cube::new(
                    Vec3::new((portal_cx + px) as f32 - 0.5, 0.5, (portal_cz + pz) as f32 - 0.5),
                    Vec3::new((portal_cx + px) as f32 + 0.5, 1.15, (portal_cz + pz) as f32 + 0.5),
                    end_portal_inside.clone()
                )));
                // Y ponemos lava justo debajo (y = 0.0) en el 3x3
                objects.push(Box::new(unit_cube((portal_cx + px) as f32, 0.0, (portal_cz + pz) as f32, lava_mat.clone())));
                continue;
            }
            // Saltamos las esquinas (las dejamos vacías como en el juego)
            if (px == -2 || px == 2) && (pz == -2 || pz == 2) {
                continue;
            }
            
            // Frame Block
            // El frame del End Portal no es un cubo completo de altura, es más chaparro (aprox 13/16 de altura)
            let frame = Cube::new(
                Vec3::new((portal_cx + px) as f32 - 0.5, 0.5, (portal_cz + pz) as f32 - 0.5),
                Vec3::new((portal_cx + px) as f32 + 0.5, 1.3125, (portal_cz + pz) as f32 + 0.5),
                end_frame_side.clone()
            )
            .with_top_material(end_frame_top.clone())
            .with_bottom_material(end_frame_bot.clone());
            objects.push(Box::new(frame));
        }
    }

    // Generación de un Mini Bosque denso para llenar la isla
    for x in -35..=35 {
        for z in -35..=35 {
            // Frecuencia de aparición de árboles usando ruido (0.97 = ~3% de probabilidad)
            if seeded_noise(seed ^ 0xABCD, x, z) > 0.97 {
                // Verificar que esté dentro de la isla (para árboles usamos is_small=false asumiendo isla principal)
                if !is_inside_island(x, z, RADIUS as f32, seed, false) { continue; }
                // Evitar el agua
                if is_lake(x, z) { continue; }
                // Evitar la entrada superior de la cueva (alrededor de x=8, z=20)
                if (x - 8).abs() <= 5 && (z - 20).abs() <= 5 { continue; }
                // Evitar puente (-2..2, -54..-34)
                if x >= -4 && x <= 4 && z >= -54 && z <= -34 { continue; }
                // Evitar colisiones con la cabaña gigante (cx=25, cz=15, radio ~16)
                if x >= 6 && x <= 45 && z >= -4 && z <= 35 { continue; }
                
                // Altura aleatoria para cada árbol (entre 4 y 6)
                let height = 4 + (seeded_noise(seed ^ 0x1234, x, z) * 3.0) as i32;
                add_tree(&mut objects, x, z, height, &wood_mat, &wood_top, &leaves_mat);
            }
        }
    }

    objects
}

/// Segundo mundo: un Bosque Carmesí del Nether independiente de la isla.
/// Comparte la seed para que sus variaciones también sean reproducibles.
pub fn build_nether_diorama(texture_atlas: &Arc<Texture>, seed: u64) -> Vec<Box<dyn RayIntersect>> {
    let mut objects: Vec<Box<dyn RayIntersect>> = Vec::new();
    let netherrack = tex_mat_from_top(texture_atlas, 7.0, 6.0);
    let soul_sand = tex_mat_from_top(texture_atlas, 8.0, 6.0);
    let glowstone = tex_mat_from_top(texture_atlas, 9.0, 6.0).with_emission(true);
    let lava = tex_mat(texture_atlas, 15.0, 0.0).with_emission(true);
    let quartz_ore = tex_mat_from_top(texture_atlas, 15.0, 1.0); // Mineral de cuarzo actualizado
    let nether_gold_ore = tex_mat_from_top(texture_atlas, 15.0, 2.0); // Mineral de oro del nether
    let gold_block = Material::new(Color::new(255, 255, 255), 80.0, [0.8, 0.5, 0.2, 0.0])
        .with_texture(Arc::clone(texture_atlas))
        .with_uv((1.0 / 16.0, 1.0 / 16.0), (7.0 / 16.0, 14.0 / 16.0)); // Bloque de oro puro
    let nether_brick = tex_mat_from_top(texture_atlas, 0.0, 14.0); // Para la fortaleza
    let obsidian = obsidian_mat(texture_atlas); // Para el portal

    // 1. Generación orgánica de la isla base (AMPLIADA)
    for x in -35..=35 {
        for z in -35..=35 {
            let base_radius = 28.0 + (x as f32 * 0.4).sin() * 4.0 + (z as f32 * 0.3).cos() * 5.0;
            let dist = ((x*x + z*z) as f32).sqrt();
            
            if dist <= base_radius {
                let depth = -25 - (seeded_noise(seed ^ 0x666, x, z) * 10.0) as i32;
                
                for y in depth..=0 {
                    // Tapering (estrechamiento hacia abajo)
                    let max_radius_at_y = base_radius + (y as f32 * 0.6);
                    if dist > max_radius_at_y { continue; }
                    
                    // Lago de lava en el centro (AMPLIADO)
                    if y >= -2 && y <= 0 && dist < 14.0 {
                        objects.push(Box::new(unit_cube(x as f32, y as f32, z as f32, lava.clone())));
                        continue;
                    }
                    
                    let material = if y == 0 {
                        if seeded_noise(seed ^ 0x111, x, z) > 0.5 { soul_sand.clone() } else { netherrack.clone() }
                    } else {
                        let rand_val = seeded_noise(seed ^ 0x222, x, y ^ z);
                        if rand_val > 0.95 {
                            quartz_ore.clone()
                        } else if rand_val > 0.90 {
                            nether_gold_ore.clone()
                        } else {
                            netherrack.clone()
                        }
                    };
                    
                    objects.push(Box::new(unit_cube(x as f32, y as f32, z as f32, material)));
                }
                
                // Estalagmitas y estructuras naturales (alejadas del centro)
                if dist > 14.0 && seeded_noise(seed ^ 0x444, x, z) > 0.96 {
                    let height = 2 + (seeded_noise(seed ^ 0x555, x, z) * 6.0) as i32;
                    for step in 1..=height {
                        objects.push(Box::new(unit_cube(x as f32, step as f32, z as f32, netherrack.clone())));
                    }
                    
                    // En la punta de algunas estalagmitas ponemos Glowstone o un Bloque de Oro!
                    let rand_tip = seeded_noise(seed, x, z);
                    if rand_tip > 0.8 {
                        objects.push(Box::new(unit_cube(x as f32, height as f32 + 1.0, z as f32, gold_block.clone())));
                    } else if rand_tip > 0.5 {
                        objects.push(Box::new(unit_cube(x as f32, height as f32 + 1.0, z as f32, glowstone.clone())));
                    }
                }
            }
        }
    }

    // 2. Puente de Fortaleza cruzando el lago de lava (4 de ancho)
    for z in -15..=15 {
        for x in -2..=1 {
            // Generamos piso con partes rotas (hoyos aleatorios)
            if seeded_noise(seed ^ 0x777, x, z) > 0.25 {
                objects.push(Box::new(unit_cube(x as f32, 1.0, z as f32, nether_brick.clone())));
                
                // Barandas en las posiciones 1 (x=-2) y 4 (x=1)
                if x == -2 || x == 1 {
                    // Las barandas también tienen probabilidad de estar rotas
                    if seeded_noise(seed ^ 0x888, x, z) > 0.15 {
                        objects.push(Box::new(unit_cube(x as f32, 2.0, z as f32, nether_brick.clone())));
                    }
                }
            }
        }
        
        // Soportes/pilares masivos que bajan hasta la lava
        if z % 6 == 0 {
            for x in &[-2, 1] { // Solo en los bordes

                // Pilar hacia abajo
                for y in -2..=0 {
                    objects.push(Box::new(unit_cube(*x as f32, y as f32, z as f32, nether_brick.clone())));
                }
            }
        }
    }

    // 3. Mini Fortaleza del Nether (al final del puente)
    for x in -7..=7 {
        for z in 15..=25 {
            // Piso de la fortaleza
            objects.push(Box::new(unit_cube(x as f32, 1.0, z as f32, nether_brick.clone())));
            
            // Paredes exteriores altas
            if x == -7 || x == 7 || z == 25 || (z == 15 && (x < -2 || x > 2)) {
                for y in 2..=6 {
                    // Ventanas (agujeros en la pared)
                    if y == 3 && z % 2 == 0 {
                        continue;
                    }
                    if y == 4 && x % 2 == 0 {
                        continue;
                    }
                    objects.push(Box::new(unit_cube(x as f32, y as f32, z as f32, nether_brick.clone())));
                }
                // Almenas superiores (coronas de la fortaleza)
                if (x + z) % 2 != 0 {
                    objects.push(Box::new(unit_cube(x as f32, 7.0, z as f32, nether_brick.clone())));
                }
            }
            
            // Pequeñas pilas de bloques de oro en el interior (reemplazando los pilares)
            if (x == -4 || x == 4) && (z == 18 || z == 22) {
                // Pilas pequeñas de oro (entre 1 y 3 bloques de alto)
                let height = if z == 18 { if x == -4 { 3 } else { 1 } } else { if x == 4 { 2 } else { 1 } };
                
                for y in 2..=1 + height {
                    objects.push(Box::new(unit_cube(x as f32, y as f32, z as f32, gold_block.clone())));
                }
            }
        }
    }

    // 4. Portal de regreso, al INICIO del puente (contrario a la fortaleza)
    add_portal(
        &mut objects,
        -2, // origin_x
        2,  // origin_y
        -15, // origin_z (inicio del puente)
        &obsidian.clone(),
        &portal_mat(texture_atlas),
    );
    
    objects
}

fn is_lake(x: i32, z: i32) -> bool {
    // Lago gigante movido a la saliente izquierda (-X)
    let dx = x as f32 - (-25.0);
    let dz = z as f32 - 15.0;
    
    let angle = dz.atan2(dx);
    let dist = (dx * dx + dz * dz).sqrt();
    
    // Deformar el radio con ondas para darle una forma muy orgánica y natural
    let wave = (angle * 3.0).sin() * 3.0 + (angle * 5.0).cos() * 2.0;
    let irregular_radius = 12.0 + wave;
    
    dist < irregular_radius
}

fn unit_cube(x: f32, y: f32, z: f32, material: Material) -> Cube {
    Cube::new(Vec3::new(x - 0.5, y - 0.5, z - 0.5), Vec3::new(x + 0.5, y + 0.5, z + 0.5), material)
}

fn add_island_surface(
    objects: &mut Vec<Box<dyn RayIntersect>>,
    center_x: i32,
    center_z: i32,
    surface_radius: i32,
    is_small: bool,
    grass_side: &Material,
    grass_top: &Material,
    dirt_mat: &Material,
    seed: u64,
) {
    for local_x in -(surface_radius+8)..=(surface_radius+8) {
        for local_z in -(surface_radius+8)..=(surface_radius+8) {
            let x = center_x + local_x;
            let z = center_z + local_z;
            
            if !is_inside_island(local_x, local_z, surface_radius as f32, seed, is_small) || is_lake(x, z) || is_cave_void(x, 0, z) {
                continue;
            }

            let block = unit_cube(x as f32, 0.0, z as f32, grass_side.clone())
                .with_top_material(grass_top.clone())
                .with_bottom_material(dirt_mat.clone());
            objects.push(Box::new(block));
        }
    }
}

fn add_voxel_island(
    objects: &mut Vec<Box<dyn RayIntersect>>,
    center_x: i32,
    center_z: i32,
    surface_radius: i32,
    is_small: bool,
    dirt: &Material,
    stone: &Material,
    seed: u64,
) {
    // Aumentar mucho la profundidad de la isla
    for y in -40..=-1 {
        // Un decaimiento de radio lento para que la isla sea gigante y ancha
        let radius_drop = match y {
            -4..=-1 => -y / 2,
            -15..=-5 => (-y as f32 * 0.4) as i32,
            -30..=-16 => (-y as f32 * 0.6) as i32,
            -40..=-31 => (-y as f32 * 0.8) as i32,
            _ => 20,
        };
        let radius = surface_radius - radius_drop;
        if radius <= 0 { continue; }

        for local_x in -(radius+8)..=(radius+8) {
            for local_z in -(radius+8)..=(radius+8) {
                let x = center_x + local_x;
                let z = center_z + local_z;
                
                if !is_inside_island(local_x, local_z, radius as f32, seed, is_small) || is_cave_void(x, y, z) {
                    continue;
                }
                
                if is_lake(x, z) {
                    let lake_depth = 3;
                    if y >= -lake_depth {
                        continue;
                    }
                }
                
                let material = if y >= -4 { dirt.clone() } else { stone.clone() };
                objects.push(Box::new(unit_cube(x as f32, y as f32, z as f32, material)));
            }
        }
    }
}

fn is_inside_island(x: i32, z: i32, base_radius: f32, seed: u64, is_small: bool) -> bool {
    let angle = (z as f32).atan2(x as f32);
    
    // Ondas y ruido para la costa
    let noise = seeded_noise(seed ^ 0x1A1A, x, z) * 2.5;
    let waves = if is_small {
        // La isla pequeña es mucho más redonda
        (angle * 3.0).sin() * 1.5 
    } else {
        // La isla grande tiene bahías y penínsulas agresivas
        (angle * 3.0).sin() * 6.0 + (angle * 5.0).cos() * 3.0
    };
    
    let irregular_radius = base_radius + noise + waves;
    let dist = ((x * x + z * z) as f32).sqrt();
    
    dist <= irregular_radius
}

fn is_cave_void(x: i32, y: i32, z: i32) -> bool {
    // Mega cueva en el fondo (Totalmente contenida dentro de la isla)
    let mut in_mega_cave = false;
    if y <= -8 && y >= -25 {
        // Radio máximo reducido (~12) para que no rompa el fondo de la isla
        let cave_radius = 12.0 
            + (y as f32 * 0.5).sin() * 2.0 
            + (x as f32 * 0.3).sin() * 3.0 
            + (z as f32 * 0.3).cos() * 3.0;
            
        let dist_to_core = ((x * x + z * z) as f32).sqrt();
        if dist_to_core <= cave_radius {
            in_mega_cave = true;
        }
    }
    
    // Entrada superior tipo sumidero
    // Ubicada en la parte frontal derecha (lejos del lago)
    let entrance_cx = 8.0;
    let entrance_cz = 20.0;
    
    let t = (-y as f32).clamp(0.0, 15.0) / 15.0; // Conecta a la cueva en y=-15
    let current_cx = entrance_cx * (1.0 - t);
    let current_cz = entrance_cz * (1.0 - t);
    
    let tunnel_radius = 3.5 + (y as f32 * 0.4).sin() * 1.0; 
    let dist_to_tunnel = ((x as f32 - current_cx).powi(2) + (z as f32 - current_cz).powi(2)).sqrt();
    let in_tunnel = y > -15 && y <= 0 && dist_to_tunnel <= tunnel_radius;
    
    // Ventana lateral de exhibición (Diorama View)
    // Corta la parte frontal de la isla (+Z) para ver hacia adentro de la cueva
    let mut in_window = false;
    if y <= -4 && y >= -28 && z >= 5 {
        // Tubo horizontal orgánico y enorme que va desde el centro hacia afuera
        // Desplazado a x=5 para evitar el lago que está en x < -10
        let window_radius = 9.0 
            + (z as f32 * 0.25).sin() * 2.0 
            + (x as f32 * 0.4).sin() * 1.5 
            + (y as f32 * 0.4).cos() * 1.5;
            
        let dist_to_window = ((x as f32 - 5.0).powi(2) + (y as f32 + 16.0).powi(2)).sqrt();
        if dist_to_window <= window_radius {
            in_window = true;
        }
    }
    
    in_mega_cave || in_tunnel || in_window
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
    // Un lago gigante en la parte frontal-izquierda de la isla (-X)
    for x in -40..=-10 {
        for z in 0..=30 {
            if is_lake(x, z) {
                let lake_depth = 3;
                
                for y in -lake_depth..=0 {
                    objects.push(Box::new(unit_cube(x as f32, y as f32, z as f32, water_mat.clone())));
                }
            }
        }
    }
}

fn add_cave(
    objects: &mut Vec<Box<dyn RayIntersect>>,
    stone_mat: &Material,
    diamond_mat: &Material,
    lava_mat: &Material,
    seed: u64,
) {
    // Generación de decoraciones dentro del vacío de la mega cueva
    for y in -25..=-8 {
        for x in -20..=20 {
            for z in -20..=30 {
                // Solo nos interesan bloques que son void (para poner cosas DENTRO de la cueva)
                if !is_cave_void(x, y, z) { 
                    // Si es pared de piedra, tal vez reemplacemos con diamante
                    if is_cave_void(x+1, y, z) || is_cave_void(x-1, y, z) || is_cave_void(x, y, z+1) || is_cave_void(x, y, z-1) {
                        if seeded_noise(seed ^ 0x9999, x, y * 10 + z) > 0.98 {
                            objects.push(Box::new(unit_cube(x as f32, y as f32, z as f32, diamond_mat.clone())));
                        }
                    }
                    continue; 
                }
                
                // Lago de lava ardiente en lo más profundo (evitando la ventana en z>10)
                if y <= -23 && z < 10 {
                    objects.push(Box::new(unit_cube(x as f32, y as f32, z as f32, lava_mat.clone())));
                    continue;
                }
                
                // Estalactitas (techo) y estalagmitas (suelo)
                let is_ceiling = !is_cave_void(x, y + 1, z);
                let is_floor = !is_cave_void(x, y - 1, z);
                
                if is_ceiling && seeded_noise(seed ^ 0xCAFE, x, z) > 0.96 {
                    // Estalactita cayendo del techo
                    let height = 2 + (seeded_noise(seed, x, z) * 3.0) as i32;
                    for step in 0..height {
                        let py = y - step;
                        if py > -23 && is_cave_void(x, py, z) {
                            objects.push(Box::new(unit_cube(x as f32, py as f32, z as f32, stone_mat.clone())));
                        }
                    }
                } else if is_floor && seeded_noise(seed ^ 0xBEEF, x, z) > 0.97 {
                    // Estalagmita subiendo del suelo
                    let height = 1 + (seeded_noise(seed, z, x) * 2.0) as i32;
                    for step in 0..height {
                        let py = y + step;
                        if py > -23 && is_cave_void(x, py, z) {
                            objects.push(Box::new(unit_cube(x as f32, py as f32, z as f32, stone_mat.clone())));
                        }
                    }
                }
            }
        }
    }
}

fn add_lodge(
    objects: &mut Vec<Box<dyn RayIntersect>>,
    texture_atlas: &Arc<Texture>,
    _stone: &Material,
    planks: &Material,
    wood: &Material,
    wood_top: &Material,
    glass: &Material,
    lamp: &Material,
) {
    // ¡La Gran Cabaña "Cozy" (Forma de Cruz Gigante)!
    // Moviendo a la saliente derecha (+X)
    let cx = 25;
    let cz = 15;
    
    // --- PISO Y PORCHE ---
    for x in (cx - 16)..=(cx + 16) {
        for z in (cz - 16)..=(cz + 16) {
            let dist_x = (x - cx as i32).abs();
            let dist_z = (z - cz as i32).abs();
            
            let inside_walls = (dist_x <= 12 && dist_z <= 6) || (dist_x <= 6 && dist_z <= 12);
            let inside_deck = (dist_x <= 15 && dist_z <= 9) || (dist_x <= 9 && dist_z <= 15);
            
            if inside_walls {
                // Piso interior de madera
                objects.push(Box::new(unit_cube(x as f32, 1.0, z as f32, planks.clone())));
            } else if inside_deck {
                // Deck exterior (porche)
                objects.push(Box::new(unit_cube(x as f32, 1.0, z as f32, wood.clone())));
                
                // Barandales del porche (postes de madera)
                let is_deck_edge = (dist_x == 15 && dist_z <= 9) || (dist_z == 15 && dist_x <= 9) ||
                                   (dist_x == 9 && dist_z >= 9 && dist_z <= 15) || 
                                   (dist_z == 9 && dist_x >= 9 && dist_x <= 15);
                
                // Entradas apuntando hacia el centro de la isla (-X y -Z)
                let is_entrance = (x == cx - 15 && z >= cz - 2 && z <= cz + 2) || 
                                  (z == cz - 15 && x >= cx - 2 && x <= cx + 2); 
                
                if is_deck_edge && !is_entrance && (x % 2 == 0 || z % 2 == 0) {
                    objects.push(Box::new(unit_cube(x as f32, 2.0, z as f32, planks.clone())));
                }
                
                // Si el deck está flotando sobre el vacío (fuera de la isla), agregar soportes hacia abajo
                if is_deck_edge && (x % 4 == 0 && z % 4 == 0) {
                    // Pilares largos de soporte para el balcón suspendido
                    for y in -10..=0 {
                        objects.push(Box::new(unit_cube(x as f32, y as f32, z as f32, wood.clone())));
                    }
                }
            }
        }
    }
    
    // --- PAREDES, VENTANAS Y TECHO ---
    for x in (cx - 15)..=(cx + 15) {
        for z in (cz - 15)..=(cz + 15) {
            let dist_x = (x - cx as i32).abs();
            let dist_z = (z - cz as i32).abs();
            
            let is_main_roof = dist_x <= 13 && dist_z <= 7;
            let is_cross_roof = dist_x <= 7 && dist_z <= 13;
            
            if !is_main_roof && !is_cross_roof {
                continue; 
            }
            
            // Altura del techo en este (x, z)
            let mut y_roof = 0;
            if is_main_roof { y_roof = y_roof.max(4 + (7 - dist_z) * 2); }
            if is_cross_roof { y_roof = y_roof.max(4 + (7 - dist_x) * 2); }
            
            // Colocar el techo
            let roof_mat = tex_mat(texture_atlas, 4.0, 15.0); // Ladrillos de piedra/teja
            objects.push(Box::new(unit_cube(x as f32, y_roof as f32, z as f32, roof_mat.clone())));
            // Para que el techo sea grueso, colocamos otro bloque debajo
            if y_roof > 4 {
                objects.push(Box::new(unit_cube(x as f32, (y_roof - 1) as f32, z as f32, wood.clone())));
            }
            
            // Colocar paredes
            let is_wall_x = dist_x == 12 && dist_z <= 6;
            let is_wall_z = dist_z == 12 && dist_x <= 6;
            let is_inner_corner = dist_x == 6 && dist_z == 6;
            
            if is_wall_x || is_wall_z || is_inner_corner {
                let is_corner = (dist_x == 12 && dist_z == 6) || (dist_z == 12 && dist_x == 6) || is_inner_corner;
                let is_window = (dist_x == 12 && dist_z <= 3) || (dist_z == 12 && dist_x <= 3);
                
                for y in 2..=(y_roof - 2) { 
                    // Chimenea gigante corta la pared en la parte trasera
                    if dist_x <= 3 && z == cz - 12 { continue; }
                    // Puerta principal
                    if dist_x <= 2 && z == cz + 6 && y <= 4 { continue; }
                    
                    if is_corner {
                        let pillar = unit_cube(x as f32, y as f32, z as f32, wood.clone())
                            .with_top_material(wood_top.clone())
                            .with_bottom_material(wood_top.clone());
                        objects.push(Box::new(pillar));
                    } else if is_window && y >= 3 && y <= 9 {
                        objects.push(Box::new(unit_cube(x as f32, y as f32, z as f32, glass.clone())));
                    } else {
                        objects.push(Box::new(unit_cube(x as f32, y as f32, z as f32, planks.clone())));
                    }
                }
            }
        }
    }
    
    // --- CHIMENEA GIGANTE DE PIEDRA ---
    // En la parte trasera: z = cz - 12. Centro en x = cx.
    let chimney_stone = tex_mat(texture_atlas, 1.0, 15.0); // Cobblestone
    for y in 1..=24 {
        for x in (cx - 3)..=(cx + 3) {
            for z in (cz - 14)..=(cz - 11) {
                let is_outer = x == cx - 3 || x == cx + 3 || z == cz - 14 || z == cz - 11;
                if is_outer {
                    // Estrechar la chimenea en la cima
                    if y > 18 && (x == cx - 3 || x == cx + 3) { continue; }
                    objects.push(Box::new(unit_cube(x as f32, y as f32, z as f32, chimney_stone.clone())));
                } else if y == 2 {
                    // Fuego (lava/lámpara) en la base
                    objects.push(Box::new(unit_cube(x as f32, y as f32, z as f32, lamp.clone())));
                }
            }
        }
    }
    
    // --- INTERIOR COZY ---
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
    
    // Aquí actualizamos la textura de libreros usando tex_mat_from_top para manejar la inversión en Y
    let bookshelf = tex_mat_from_top(texture_atlas, 3.0, 2.0);
    
    // Mesa de crafteo y hornos cerca de la chimenea
    objects.push(Box::new(unit_cube((cx + 2) as f32, 2.0, (cz - 9) as f32, crafting_side.clone())
        .with_top_material(crafting_top).with_front_material(crafting_front.clone())
        .with_back_material(crafting_front).with_left_material(crafting_side.clone())
        .with_right_material(crafting_side)));
        
    objects.push(Box::new(unit_cube(cx as f32, 2.0, (cz - 10) as f32, furnace_side.clone())
        .with_top_material(furnace_top.clone()).with_front_material(furnace_front.clone())
        .with_back_material(furnace_front.clone()).with_left_material(furnace_side.clone())
        .with_right_material(furnace_side.clone())));
        
    objects.push(Box::new(unit_cube((cx - 1) as f32, 2.0, (cz - 10) as f32, furnace_side.clone())
        .with_top_material(furnace_top).with_front_material(furnace_front.clone())
        .with_back_material(furnace_front).with_left_material(furnace_side.clone())
        .with_right_material(furnace_side)));

    // Camas en el ala este (-X)
    for z in (cz - 1)..=(cz + 1) {
        objects.push(Box::new(unit_cube((cx - 10) as f32, 2.0, z as f32, bed_foot_side.clone())
            .with_top_material(bed_foot_top.clone()).with_front_material(bed_foot_front.clone())
            .with_back_material(bed_foot_front.clone()).with_left_material(bed_foot_side.clone())
            .with_right_material(bed_foot_side.clone())));
        objects.push(Box::new(unit_cube((cx - 11) as f32, 2.0, z as f32, bed_head_side.clone())
            .with_top_material(bed_head_top.clone()).with_back_material(bed_head_back.clone())
            .with_left_material(bed_head_side.clone()).with_right_material(bed_head_side.clone())));
    }
    
    // Grandes paredes de libreros
    for y in 2..=8 {
        objects.push(Box::new(unit_cube((cx + 4) as f32, y as f32, (cz + 2) as f32, bookshelf.clone())));
        objects.push(Box::new(unit_cube((cx - 4) as f32, y as f32, (cz + 2) as f32, bookshelf.clone())));
        objects.push(Box::new(unit_cube((cx + 4) as f32, y as f32, (cz - 2) as f32, bookshelf.clone())));
        objects.push(Box::new(unit_cube((cx - 4) as f32, y as f32, (cz - 2) as f32, bookshelf.clone())));
    }
    
    // Iluminación interior (Gran Candelabro)
    objects.push(Box::new(unit_cube(cx as f32, 14.0, cz as f32, wood.clone())));
    objects.push(Box::new(unit_cube(cx as f32, 13.0, cz as f32, lamp.clone())));
    objects.push(Box::new(unit_cube((cx + 1) as f32, 13.0, cz as f32, lamp.clone())));
    objects.push(Box::new(unit_cube((cx - 1) as f32, 13.0, cz as f32, lamp.clone())));
    objects.push(Box::new(unit_cube(cx as f32, 13.0, (cz + 1) as f32, lamp.clone())));
    objects.push(Box::new(unit_cube(cx as f32, 13.0, (cz - 1) as f32, lamp.clone())));
}

fn add_portal(objects: &mut Vec<Box<dyn RayIntersect>>, origin_x: i32, origin_y: i32, origin_z: i32, obsidian: &Material, portal: &Material) {
    // Marco exterior 4x6; interior morado 2x4.
    for x in origin_x..=origin_x + 3 {
        objects.push(Box::new(unit_cube(x as f32, origin_y as f32, origin_z as f32, obsidian.clone())));
        objects.push(Box::new(unit_cube(x as f32, (origin_y + 5) as f32, origin_z as f32, obsidian.clone())));
    }
    for y in (origin_y + 1)..=(origin_y + 4) {
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

fn add_portal_structure(
    objects: &mut Vec<Box<dyn RayIntersect>>,
    cx: i32,
    cz: i32,
    stone_brick: &Material,
    glowstone: &Material,
) {
    // La plataforma debe ser simétrica. El portal va de x=cx a x=cx+3.
    // Damos un borde de 2 bloques a cada lado en X: de cx-2 a cx+5.
    // Damos un borde de 3 bloques en Z: de cz-3 a cz+3.
    for x in (cx - 2)..=(cx + 5) {
        for z in (cz - 3)..=(cz + 3) {
            objects.push(Box::new(unit_cube(x as f32, 1.0, z as f32, stone_brick.clone())));
            
            // Pilares en las esquinas
            let is_corner = (x == cx - 2 || x == cx + 5) && (z == cz - 3 || z == cz + 3);
            if is_corner {
                // Pilar de altura 3 (niveles 2, 3, 4)
                objects.push(Box::new(unit_cube(x as f32, 2.0, z as f32, stone_brick.clone())));
                objects.push(Box::new(unit_cube(x as f32, 3.0, z as f32, stone_brick.clone())));
                objects.push(Box::new(unit_cube(x as f32, 4.0, z as f32, stone_brick.clone())));
                // Glowstone en la punta (nivel 5)
                objects.push(Box::new(unit_cube(x as f32, 5.0, z as f32, glowstone.clone())));
            }
        }
    }
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

pub fn build_end_diorama(texture_atlas: &Arc<Texture>, seed: u64) -> Vec<Box<dyn RayIntersect>> {
    let mut objects: Vec<Box<dyn RayIntersect>> = Vec::new();

    let end_stone_mat = tex_mat_from_top(texture_atlas, 15.0, 10.0);
    let obsidian = obsidian_mat(texture_atlas);
    
    // Isla Principal del End
    const END_RADIUS: i32 = 45;
    add_island_surface(&mut objects, 0, 0, END_RADIUS, false, &end_stone_mat, &end_stone_mat, &end_stone_mat, seed);
    add_voxel_island(&mut objects, 0, 0, END_RADIUS, false, &end_stone_mat, &end_stone_mat, seed);
    
    // Pilares de Obsidiana en un círculo
    let num_pillars = 8;
    for i in 0..num_pillars {
        let angle = (i as f32 / num_pillars as f32) * std::f32::consts::PI * 2.0;
        let radius = 25.0;
        let px = (angle.cos() * radius).round() as i32;
        let pz = (angle.sin() * radius).round() as i32;
        
        let height = 15 + (seeded_noise(seed + i, px, pz) * 15.0) as i32;
        
        for y in 1..=height {
            for x in -1..=1 {
                for z in -1..=1 {
                    objects.push(Box::new(unit_cube((px + x) as f32, y as f32, (pz + z) as f32, obsidian.clone())));
                }
            }
        }
        
        // Un "cristal" simulado (Bedrock/Glowstone/Glass) arriba
        let glass_mat = Material::new(Color::new(255, 100, 255), 50.0, [0.1, 0.4, 0.1, 0.8])
            .with_refractive_index(1.5).with_emission(true);
        objects.push(Box::new(unit_cube(px as f32, (height + 1) as f32, pz as f32, glass_mat)));
    }
    
    // Portal de salida central (Bedrock y End Portal)
    let bedrock = tex_mat_from_top(texture_atlas, 1.0, 1.0); // Coordenada de bedrock clásica (o piedra en su defecto)
    let end_portal_inside = tex_mat_from_top(texture_atlas, 15.0, 12.0)
        .with_emission(true)
        .with_portal(true);
        
    for x in -2i32..=2 {
        for z in -2i32..=2 {
            if x.abs() == 2 || z.abs() == 2 {
                objects.push(Box::new(unit_cube(x as f32, 1.0, z as f32, bedrock.clone())));
            } else if x == 0 && z == 0 {
                // Pilar central de bedrock
                objects.push(Box::new(unit_cube(0.0, 1.0, 0.0, bedrock.clone())));
                objects.push(Box::new(unit_cube(0.0, 2.0, 0.0, bedrock.clone())));
                objects.push(Box::new(unit_cube(0.0, 3.0, 0.0, bedrock.clone())));
            } else {
                // Bloques de portal del end 
                objects.push(Box::new(Cube::new(
                    Vec3::new(x as f32 - 0.5, 0.5, z as f32 - 0.5),
                    Vec3::new(x as f32 + 0.5, 1.15, z as f32 + 0.5),
                    end_portal_inside.clone()
                )));
            }
        }
    }
    
    objects
}
