use crate::framebuffer::Framebuffer;
use crate::ray_intersect::Material;

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

                // Slot borders
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
