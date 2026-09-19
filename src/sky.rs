use crate::color::Color;
use nalgebra_glm::Vec3;

pub fn hash2(x: f32, y: f32) -> f32 {
    let seed = x * 12.9898 + y * 78.233;
    (seed.sin() * 43758.5453).fract().abs()
}

pub fn noise(x: f32, z: f32) -> f32 {
    let ix = x.floor();
    let iz = z.floor();
    let fx = x - ix; // x.fract() in Rust preserves sign, which breaks noise for negative coords!
    let fz = z - iz;
    
    let a = hash2(ix, iz);
    let b = hash2(ix + 1.0, iz);
    let c = hash2(ix, iz + 1.0);
    let d = hash2(ix + 1.0, iz + 1.0);
    
    let ux = fx * fx * (3.0 - 2.0 * fx);
    let uz = fz * fz * (3.0 - 2.0 * fz);
    
    a * (1.0 - ux) + b * ux + (c - a) * uz * (1.0 - ux) + (d - b) * ux * uz
}

pub fn get_sky_color(ray_direction: &Vec3, time_of_day: f32) -> Color {
    let day_zenith = Color::new(80, 150, 255);
    let day_horizon = Color::new(180, 220, 255);
    
    let night_zenith = Color::new(2, 2, 10);
    let night_horizon = Color::new(10, 15, 30);
    
    let sunset_horizon = Color::new(255, 120, 50);

    let t_y = ray_direction.y.max(0.0);
    let angle = time_of_day * std::f32::consts::PI * 2.0 - std::f32::consts::PI / 2.0;
    let sun_height = angle.sin();
    
    let day_factor = ((sun_height + 0.2) * 2.0).clamp(0.0, 1.0);
    let sunset_factor = (1.0 - sun_height.abs() * 3.0).clamp(0.0, 1.0);
    
    let current_zenith = night_zenith * (1.0 - day_factor) + day_zenith * day_factor;
    let base_horizon = night_horizon * (1.0 - day_factor) + day_horizon * day_factor;
    let current_horizon = base_horizon * (1.0 - sunset_factor) + sunset_horizon * sunset_factor;
    
    let mut sky = current_horizon * (1.0 - t_y) + current_zenith * t_y;

    // Sun and Moon (Minecraft style: square and hidden below horizon)
    if ray_direction.y > -0.05 {
        let sun_x = angle.cos() * 10.0;
        let sun_y = angle.sin() * 10.0;
        let sun_dir = nalgebra_glm::normalize(&Vec3::new(sun_x, sun_y, 8.0));
        let moon_dir = nalgebra_glm::normalize(&Vec3::new(-sun_x, -sun_y, -8.0));

        let sun_dz = nalgebra_glm::dot(ray_direction, &sun_dir);
        if sun_dz > 0.95 {
            let sun_right = nalgebra_glm::normalize(&nalgebra_glm::cross(&sun_dir, &Vec3::new(0.0, 0.0, 1.0)));
            let sun_up = nalgebra_glm::normalize(&nalgebra_glm::cross(&sun_right, &sun_dir));
            let sun_dx = nalgebra_glm::dot(ray_direction, &sun_right);
            let sun_dy = nalgebra_glm::dot(ray_direction, &sun_up);
            
            let scale = 1.0 / sun_dz;
            let px = sun_dx * scale;
            let py = sun_dy * scale;
            
            if px.abs() < 0.065 && py.abs() < 0.065 {
                if px.abs() < 0.06 && py.abs() < 0.06 {
                    sky = Color::new(255, 255, 220); // Sun core
                } else {
                    sky = sky * 0.5 + Color::new(255, 255, 200) * 0.5; // Sun border
                }
            }
        }

        let moon_dz = nalgebra_glm::dot(ray_direction, &moon_dir);
        if moon_dz > 0.95 {
            let moon_right = nalgebra_glm::normalize(&nalgebra_glm::cross(&moon_dir, &Vec3::new(0.0, 0.0, 1.0)));
            let moon_up = nalgebra_glm::normalize(&nalgebra_glm::cross(&moon_right, &moon_dir));
            let moon_dx = nalgebra_glm::dot(ray_direction, &moon_right);
            let moon_dy = nalgebra_glm::dot(ray_direction, &moon_up);
            
            let scale = 1.0 / moon_dz;
            let px = moon_dx * scale;
            let py = moon_dy * scale;
            
            if px.abs() < 0.055 && py.abs() < 0.055 {
                if px.abs() < 0.05 && py.abs() < 0.05 {
                    sky = Color::new(220, 220, 255); // Moon core
                } else {
                    sky = sky * 0.5 + Color::new(200, 200, 255) * 0.5; // Moon border
                }
            }
        }
    }

    // Stars (medium density)
    if day_factor < 0.2 {
        let qx = (ray_direction.x * 500.0).round();
        let qy = (ray_direction.y * 500.0).round();
        let qz = (ray_direction.z * 500.0).round();
        
        let seed = qx * 12.9898 + qy * 78.233 + qz * 37.719;
        let hash = (seed.sin() * 43758.5453).fract().abs();
        
        if hash > 0.997 {
            let star_brightness = (hash - 0.997) * 400.0;
            let visibility = (1.0 - day_factor * 5.0).clamp(0.0, 1.0);
            sky = sky + Color::new(255, 255, 255) * (star_brightness * visibility);
        }
    }

    // Cosmos / Milky Way (Advanced Algorithmic Art mimicking the reference image)
    if day_factor < 0.4 {
        // Milky Way stretches across the sky
        let galactic_pole = nalgebra_glm::normalize(&Vec3::new(0.6, 0.4, 0.5));
        let galactic_equator_dot = nalgebra_glm::dot(ray_direction, &galactic_pole);
        
        let offset = time_of_day * 0.05; // Extremely slow movement for cosmic scale
        
        // 3D-to-2D projection for seamless noise
        let nx = ray_direction.x;
        let ny = ray_direction.y;
        let nz = ray_direction.z;
        
        // Base structure noise to warp the galaxy band (makes it twist organically)
        // Lower frequency (1.0) and higher amplitude (0.8) for massive sweeping bends
        let base_warp = noise(nx * 1.0 + offset, nz * 1.0 - offset);
        let band_dist = (galactic_equator_dot + (base_warp - 0.5) * 0.8).abs();
        
        // The galaxy is thickest at the warped equator. 
        // Lowered multiplier (1.5) makes the band wider and softer.
        let galaxy_band = (1.0 - band_dist * 1.5).clamp(0.0, 1.0);
        
        // A strictly thinner band just for the core to prevent the "laser" effect at the ends.
        // We add high-frequency noise here so the core isn't a perfectly straight line!
        let core_noise = noise(nx * 8.0, nz * 8.0);
        let core_dist = (band_dist + (core_noise - 0.5) * 0.15).abs();
        let core_band = (1.0 - core_dist * 5.0).clamp(0.0, 1.0);
        
        if galaxy_band > 0.0 {
            // High-detail Fractal Brownian Motion (FBM) for complex dust clouds and nebulas
            let n1_xy = noise(nx * 1.5 + offset, ny * 1.5 - offset);
            let n1_zy = noise(nz * 1.5 - offset, ny * 1.5 + offset);
            let n1 = (n1_xy + n1_zy) * 0.5;
            
            let n2_xy = noise(nx * 3.0 - offset, ny * 3.0 + offset);
            let n2_zy = noise(nz * 3.0 + offset, ny * 3.0 - offset);
            let n2 = (n2_xy + n2_zy) * 0.25;
            
            let n3_xy = noise(nx * 6.0, nz * 6.0);
            let n3 = n3_xy * 0.125;
            
            let n4_xy = noise(nx * 12.0, ny * 12.0);
            let n4 = n4_xy * 0.0625;
            
            let fbm = n1 + n2 + n3 + n4; // Range ~ 0.0 to 0.9375
            
            // Create dark rifts and glowing edges by squaring the FBM to stretch the dark gaps
            // Increased multiplier slightly to ensure dense clumps reach maximum intensity
            let dust_clump = (fbm.powi(2) * 2.5 - 0.1).clamp(0.0, 1.0);
            
            let night_fade = (1.0 - day_factor * 2.5).clamp(0.0, 1.0);
            // Removed powi(2) from galaxy_band to soften the hard line and make the fade much smoother
            let intensity = galaxy_band * dust_clump * night_fade;
            
            if intensity > 0.01 {
                let bg_color = Color::new(20, 5, 30); // Very dark space background
                let purple_color = Color::new(130, 20, 150); // Vivid purple for the fringes
                let dark_blue = Color::new(5, 20, 60); // Much darker blue transition
                let mid_blue = Color::new(15, 50, 100); // Darker standard blue nebula (less "claro")
                let core_color = Color::new(170, 210, 255); // Soft cyan/white center
                
                // Secondary noise layer to dictate where the purple gas clouds are
                let purple_noise = noise(nx * 2.5 + offset, ny * 2.5 - offset);
                
                let mut dust_color = bg_color;
                
                // 1. Purple fringes at the very edges (lowest intensity)
                if intensity > 0.02 {
                    let t = ((intensity - 0.02) * 8.0).clamp(0.0, 1.0);
                    let active_edge_color = if purple_noise > 0.4 {
                        let p_blend = ((purple_noise - 0.4) * 2.0).clamp(0.0, 1.0);
                        bg_color * (1.0 - p_blend) + purple_color * p_blend
                    } else {
                        bg_color
                    };
                    dust_color = dust_color * (1.0 - t) + active_edge_color * t;
                }
                
                // 2. Transition into dark blue
                if intensity > 0.15 {
                    let t = ((intensity - 0.15) * 4.0).clamp(0.0, 1.0);
                    dust_color = dust_color * (1.0 - t) + dark_blue * t;
                }
                
                // 3. Transition into mid blue (Higher threshold = much smaller light blue area)
                if intensity > 0.45 {
                    let t = ((intensity - 0.45) * 3.0).clamp(0.0, 1.0);
                    dust_color = dust_color * (1.0 - t) + mid_blue * t;
                }
                
                // 4. Transition into bright core (Constrained by core_band so it's physically thin)
                if intensity > 0.7 {
                    // Multiplying by core_band forces the white/cyan color to ONLY exist in the 
                    // exact center line of the galaxy, completely breaking up the thick "laser" look.
                    let t = ((intensity - 0.7) * 4.0).clamp(0.0, 1.0) * core_band;
                    dust_color = dust_color * (1.0 - t) + core_color * t;
                }
                
                // Embedded dense tiny stars within the galaxy structure
                let qx = (nx * 800.0).round();
                let qy = (ny * 800.0).round();
                let qz = (nz * 800.0).round();
                let star_seed = qx * 12.9898 + qy * 78.233 + qz * 37.719;
                let star_hash = (star_seed.sin() * 43758.5453).fract().abs();
                
                if star_hash > 0.985 { 
                    // High density inside the milky way core, softer glow
                    let star_glow = (star_hash - 0.985) * 200.0 * intensity;
                    sky = sky + Color::new(255, 255, 255) * star_glow;
                }
                
                // Additive blend for glowing stardust clouds
                sky = sky + dust_color * (intensity * 1.2);
            }
        }
    }

    // Clouds
    if day_factor > 0.0 && t_y > 0.0 {
        let t_y_clamped = t_y.max(0.01);
        let cloud_height = 100.0;
        let offset = time_of_day * 1000.0; // Mover el bloque físico
        
        let cx = (ray_direction.x / t_y_clamped * cloud_height + offset).floor();
        let cz = (ray_direction.z / t_y_clamped * cloud_height).floor();
        
        let n = noise(cx * 0.02, cz * 0.02);
        
        if n > 0.6 { // Threshold for clouds
            let cloud_color = Color::new(255, 255, 255);
            let cloud_alpha = (n - 0.6) * 5.0; // Edge smoothing
            let distance_fade = (t_y * 5.0).clamp(0.0, 1.0); // Fade out near horizon instead of sharp cut
            let cloud_alpha = cloud_alpha.clamp(0.0, 0.9) * day_factor * distance_fade;
            
            sky = sky * (1.0 - cloud_alpha) + cloud_color * cloud_alpha;
        }
    }

    sky
}
