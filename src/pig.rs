use crate::cube::Cube;
use crate::ray_intersect::{Intersect, Material, RayIntersect};
use crate::texture::Texture;
use nalgebra_glm::Vec3;
use std::sync::Arc;

pub struct Pig {
    pub position: Vec3,
    pub rotation: f32, // angle in radians around Y axis
    pub time: f32,
    pub leg_angle: f32,

    body: Cube,
    head: Cube,
    leg_fl: Cube,
    leg_fr: Cube,
    leg_bl: Cube,
    leg_br: Cube,
}

fn rotate_y(v: &Vec3, angle: f32) -> Vec3 {
    let c = angle.cos();
    let s = angle.sin();
    Vec3::new(v.x * c + v.z * s, v.y, -v.x * s + v.z * c)
}

fn rotate_x(v: &Vec3, angle: f32) -> Vec3 {
    let c = angle.cos();
    let s = angle.sin();
    Vec3::new(v.x, v.y * c - v.z * s, v.y * s + v.z * c)
}

fn skin_mat(atlas: &Arc<Texture>, x: f32, y: f32, w: f32, h: f32) -> Material {
    let scale_u = w / 64.0;
    let scale_v = h / 64.0;
    let offset_u = x / 64.0;
    let offset_v = 1.0 - (y + h) / 64.0;
    Material::new(
        crate::color::Color::new(255, 255, 255),
        0.0,
        [0.8, 0.1, 0.0, 0.0],
    )
    .with_texture(Arc::clone(atlas))
    .with_uv((scale_u, scale_v), (offset_u, offset_v))
}

fn map_cube(
    min: Vec3,
    max: Vec3,
    atlas: &Arc<Texture>,
    top: (f32, f32, f32, f32),
    bottom: (f32, f32, f32, f32),
    front: (f32, f32, f32, f32),
    back: (f32, f32, f32, f32),
    left: (f32, f32, f32, f32),
    right: (f32, f32, f32, f32),
) -> Cube {
    let base = skin_mat(atlas, front.0, front.1, front.2, front.3);
    let mut c = Cube::new(min, max, base);
    c = c.with_top_material(skin_mat(atlas, top.0, top.1, top.2, top.3));
    c = c.with_bottom_material(skin_mat(atlas, bottom.0, bottom.1, bottom.2, bottom.3));
    c = c.with_front_material(skin_mat(atlas, front.0, front.1, front.2, front.3)); // +Z
    c = c.with_back_material(skin_mat(atlas, back.0, back.1, back.2, back.3)); // -Z
    c = c.with_left_material(skin_mat(atlas, left.0, left.1, left.2, left.3)); // -X
    c = c.with_right_material(skin_mat(atlas, right.0, right.1, right.2, right.3)); // +X
    c
}

impl Pig {
    pub fn new(position: Vec3, atlas: &Arc<Texture>) -> Self {
        // Body: 10x16x8 pixels (W x L x H)
        let body = map_cube(
            Vec3::new(-5.0 / 16.0, 6.0 / 16.0, -8.0 / 16.0),
            Vec3::new(5.0 / 16.0, 14.0 / 16.0, 8.0 / 16.0),
            atlas,
            (36.0, 16.0, 10.0, 16.0), // Top
            (46.0, 16.0, 10.0, 16.0), // Bottom
            (36.0, 8.0, 10.0, 8.0),   // Front (+Z)
            (46.0, 8.0, 10.0, 8.0),   // Back (-Z)
            (46.0, 16.0, 8.0, 16.0),  // Left (-X)
            (28.0, 16.0, 8.0, 16.0),  // Right (+X)
        );

        // Head: 8x8x8 pixels
        let head = map_cube(
            Vec3::new(-4.0 / 16.0, 8.0 / 16.0, 8.0 / 16.0),
            Vec3::new(4.0 / 16.0, 16.0 / 16.0, 16.0 / 16.0),
            atlas,
            (8.0, 0.0, 8.0, 8.0),  // Top
            (16.0, 0.0, 8.0, 8.0), // Bottom
            (8.0, 8.0, 8.0, 8.0),  // Front (+Z)
            (24.0, 8.0, 8.0, 8.0), // Back (-Z)
            (16.0, 8.0, 8.0, 8.0), // Left (-X)
            (0.0, 8.0, 8.0, 8.0),  // Right (+X)
        );

        // Legs: 4x6x4 pixels
        let leg_top = (4.0, 16.0, 4.0, 4.0);
        let leg_bottom = (8.0, 16.0, 4.0, 4.0);
        let leg_front = (4.0, 20.0, 4.0, 6.0);
        let leg_back = (12.0, 20.0, 4.0, 6.0);
        let leg_left = (8.0, 20.0, 4.0, 6.0);
        let leg_right = (0.0, 20.0, 4.0, 6.0);

        let leg_fl = map_cube(
            Vec3::new(1.0 / 16.0, 0.0, 4.0 / 16.0),
            Vec3::new(5.0 / 16.0, 6.0 / 16.0, 8.0 / 16.0),
            atlas,
            leg_top,
            leg_bottom,
            leg_front,
            leg_back,
            leg_left,
            leg_right,
        );
        let leg_fr = map_cube(
            Vec3::new(-5.0 / 16.0, 0.0, 4.0 / 16.0),
            Vec3::new(-1.0 / 16.0, 6.0 / 16.0, 8.0 / 16.0),
            atlas,
            leg_top,
            leg_bottom,
            leg_front,
            leg_back,
            leg_left,
            leg_right,
        );
        let leg_bl = map_cube(
            Vec3::new(1.0 / 16.0, 0.0, -7.0 / 16.0),
            Vec3::new(5.0 / 16.0, 6.0 / 16.0, -3.0 / 16.0),
            atlas,
            leg_top,
            leg_bottom,
            leg_front,
            leg_back,
            leg_left,
            leg_right,
        );
        let leg_br = map_cube(
            Vec3::new(-5.0 / 16.0, 0.0, -7.0 / 16.0),
            Vec3::new(-1.0 / 16.0, 6.0 / 16.0, -3.0 / 16.0),
            atlas,
            leg_top,
            leg_bottom,
            leg_front,
            leg_back,
            leg_left,
            leg_right,
        );

        Pig {
            position,
            rotation: 0.0,
            time: 0.0,
            leg_angle: 0.0,
            body,
            head,
            leg_fl,
            leg_fr,
            leg_bl,
            leg_br,
        }
    }
}

impl RayIntersect for Pig {
    fn ray_intersect(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Option<Intersect> {
        let local_origin = rotate_y(&(ray_origin - self.position), -self.rotation);
        let local_direction = rotate_y(ray_direction, -self.rotation);

        let mut closest_intersect: Option<Intersect> = None;

        let mut check = |cube: &Cube, angle: f32, pivot: Vec3| {
            let o = rotate_x(&(local_origin - pivot), -angle) + pivot;
            let d = rotate_x(&local_direction, -angle);
            if let Some(mut intersect) = cube.ray_intersect(&o, &d) {
                if closest_intersect
                    .as_ref()
                    .is_none_or(|c| intersect.distance < c.distance)
                {
                    intersect.point = pivot + rotate_x(&(intersect.point - pivot), angle);
                    intersect.normal = rotate_x(&intersect.normal, angle);
                    closest_intersect = Some(intersect);
                }
            }
        };

        check(&self.body, 0.0, Vec3::zeros());
        check(&self.head, 0.0, Vec3::zeros());

        // Pivots for the legs are around their top: y = 6.0/16.0
        // And centered on their respective Z coordinates to prevent orbiting!
        let pivot_front = Vec3::new(0.0, 6.0 / 16.0, 6.0 / 16.0);
        let pivot_back = Vec3::new(0.0, 6.0 / 16.0, -5.0 / 16.0);

        check(&self.leg_fl, self.leg_angle, pivot_front);
        check(&self.leg_br, self.leg_angle, pivot_back);
        check(&self.leg_fr, -self.leg_angle, pivot_front);
        check(&self.leg_bl, -self.leg_angle, pivot_back);

        if let Some(mut intersect) = closest_intersect {
            intersect.point = self.position + rotate_y(&intersect.point, self.rotation);
            intersect.normal = rotate_y(&intersect.normal, self.rotation);
            Some(intersect)
        } else {
            None
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn update(&mut self, _time: f32) {
        self.time += 0.05;

        // Circular movement around (2, 2)
        // velocity = (-sin, cos). atan2(-sin, cos) = -time.
        self.rotation = -self.time * 0.5;
        self.position.x = 2.0 + (self.time * 0.5).cos() * 3.0;
        self.position.z = 2.0 + (self.time * 0.5).sin() * 3.0;

        // Walking animation for legs
        self.leg_angle = (self.time * 5.0).sin() * 0.5;
    }
}
