#![allow(dead_code)]
use crate::ray_intersect::{Intersect, Material, RayIntersect};
use nalgebra_glm::Vec3;

#[derive(Clone)]
pub struct Cube {
    pub min: Vec3,
    pub max: Vec3,
    pub material: Material,
    pub top_material: Option<Material>,
    pub bottom_material: Option<Material>,
    pub front_material: Option<Material>,
    pub back_material: Option<Material>,
    pub left_material: Option<Material>,
    pub right_material: Option<Material>,
}

impl Cube {
    pub fn new(min: Vec3, max: Vec3, material: Material) -> Self {
        Cube { 
            min, max, material, 
            top_material: None,
            bottom_material: None,
            front_material: None,
            back_material: None,
            left_material: None,
            right_material: None,
        }
    }

    pub fn with_top_material(mut self, top_mat: Material) -> Self {
        self.top_material = Some(top_mat);
        self
    }
    pub fn with_bottom_material(mut self, bottom_mat: Material) -> Self {
        self.bottom_material = Some(bottom_mat);
        self
    }
    pub fn with_front_material(mut self, front_mat: Material) -> Self {
        self.front_material = Some(front_mat);
        self
    }
    pub fn with_back_material(mut self, back_mat: Material) -> Self {
        self.back_material = Some(back_mat);
        self
    }
    pub fn with_left_material(mut self, left_mat: Material) -> Self {
        self.left_material = Some(left_mat);
        self
    }
    pub fn with_right_material(mut self, right_mat: Material) -> Self {
        self.right_material = Some(right_mat);
        self
    }
}

impl RayIntersect for Cube {
    fn ray_intersect(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Option<Intersect> {
        let mut t_near = f32::NEG_INFINITY;
        let mut t_far = f32::INFINITY;
        let mut near_axis = 0;
        let mut near_sign = 1.0;

        for i in 0..3 {
            let origin = ray_origin[i];
            let dir = ray_direction[i];
            let min_val = self.min[i];
            let max_val = self.max[i];

            if dir.abs() < 1e-8 {
                if origin < min_val || origin > max_val {
                    return None;
                }
            } else {
                let inv_dir = 1.0 / dir;
                let mut t1 = (min_val - origin) * inv_dir;
                let mut t2 = (max_val - origin) * inv_dir;
                let mut sign = -1.0;

                if t1 > t2 {
                    std::mem::swap(&mut t1, &mut t2);
                    sign = 1.0;
                }

                if t1 > t_near {
                    t_near = t1;
                    near_axis = i;
                    near_sign = sign;
                }

                if t2 < t_far {
                    t_far = t2;
                }

                if t_near > t_far || t_far < 0.0 {
                    return None;
                }
            }
        }

        let distance = if t_near > 0.0 {
            t_near
        } else if t_far > 0.0 {
            t_far
        } else {
            return None;
        };

        let point = ray_origin + ray_direction * distance;

        let mut normal = Vec3::zeros();
        if distance == t_near {
            normal[near_axis] = near_sign;
        } else {
            let bias = 1e-4;
            if (point.x - self.min.x).abs() < bias {
                normal = Vec3::new(1.0, 0.0, 0.0);
            } else if (point.x - self.max.x).abs() < bias {
                normal = Vec3::new(-1.0, 0.0, 0.0);
            } else if (point.y - self.min.y).abs() < bias {
                normal = Vec3::new(0.0, 1.0, 0.0);
            } else if (point.y - self.max.y).abs() < bias {
                normal = Vec3::new(0.0, -1.0, 0.0);
            } else if (point.z - self.min.z).abs() < bias {
                normal = Vec3::new(0.0, 0.0, 1.0);
            } else {
                normal = Vec3::new(0.0, 0.0, -1.0);
            }
        }

        let size = self.max - self.min;
        let (u, v) = if normal.x.abs() > 0.5 {
            if normal.x > 0.0 {
                ((self.max.z - point.z) / size.z, (point.y - self.min.y) / size.y)
            } else {
                ((point.z - self.min.z) / size.z, (point.y - self.min.y) / size.y)
            }
        } else if normal.y.abs() > 0.5 {
            if normal.y > 0.0 {
                ((point.x - self.min.x) / size.x, (self.max.z - point.z) / size.z)
            } else {
                ((point.x - self.min.x) / size.x, (point.z - self.min.z) / size.z)
            }
        } else {
            if normal.z > 0.0 {
                ((point.x - self.min.x) / size.x, (point.y - self.min.y) / size.y)
            } else {
                ((self.max.x - point.x) / size.x, (point.y - self.min.y) / size.y)
            }
        };

        let material = if normal.y > 0.5 {
            self.top_material.as_ref().unwrap_or(&self.material).clone()
        } else if normal.y < -0.5 {
            self.bottom_material.as_ref().unwrap_or(&self.material).clone()
        } else if normal.z > 0.5 {
            self.front_material.as_ref().unwrap_or(&self.material).clone() // +Z
        } else if normal.z < -0.5 {
            self.back_material.as_ref().unwrap_or(&self.material).clone()  // -Z
        } else if normal.x > 0.5 {
            self.right_material.as_ref().unwrap_or(&self.material).clone() // +X
        } else if normal.x < -0.5 {
            self.left_material.as_ref().unwrap_or(&self.material).clone()  // -X
        } else {
            self.material.clone()
        };

        Some(Intersect {
            point,
            normal,
            distance,
            material,
            u: u.clamp(0.0, 1.0),
            v: v.clamp(0.0, 1.0),
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
