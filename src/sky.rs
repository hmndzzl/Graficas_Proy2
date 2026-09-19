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
