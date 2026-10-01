use crate::cube::Cube;
use crate::ray_intersect::{Intersect, RayIntersect};
use nalgebra_glm::Vec3;
use std::collections::HashMap;

#[derive(Clone)]
pub struct VoxelGrid {
    pub blocks: HashMap<[i32; 3], Cube>,
}

impl VoxelGrid {
    pub fn new() -> Self {
        VoxelGrid {
            blocks: HashMap::new(),
        }
    }

    pub fn insert(&mut self, x: i32, y: i32, z: i32, cube: Cube) {
        self.blocks.insert([x, y, z], cube);
    }

    pub fn remove(&mut self, x: i32, y: i32, z: i32) {
        self.blocks.remove(&[x, y, z]);
    }
}

impl RayIntersect for VoxelGrid {
    fn ray_intersect(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Option<Intersect> {
        let mut current_voxel = [
            ray_origin.x.round() as i32,
            ray_origin.y.round() as i32,
            ray_origin.z.round() as i32,
        ];

        let step_x = if ray_direction.x > 0.0 { 1 } else { -1 };
        let step_y = if ray_direction.y > 0.0 { 1 } else { -1 };
        let step_z = if ray_direction.z > 0.0 { 1 } else { -1 };

        // Voxel boundaries are at half-integers because cubes are centered on integers
        let voxel_border_x = current_voxel[0] as f32 + (step_x as f32) * 0.5;
        let voxel_border_y = current_voxel[1] as f32 + (step_y as f32) * 0.5;
        let voxel_border_z = current_voxel[2] as f32 + (step_z as f32) * 0.5;

        let mut t_max_x = if ray_direction.x.abs() > 1e-6 { (voxel_border_x - ray_origin.x) / ray_direction.x } else { f32::MAX };
        let mut t_max_y = if ray_direction.y.abs() > 1e-6 { (voxel_border_y - ray_origin.y) / ray_direction.y } else { f32::MAX };
        let mut t_max_z = if ray_direction.z.abs() > 1e-6 { (voxel_border_z - ray_origin.z) / ray_direction.z } else { f32::MAX };

        let t_delta_x = if ray_direction.x.abs() > 1e-6 { (1.0 / ray_direction.x).abs() } else { f32::MAX };
        let t_delta_y = if ray_direction.y.abs() > 1e-6 { (1.0 / ray_direction.y).abs() } else { f32::MAX };
        let t_delta_z = if ray_direction.z.abs() > 1e-6 { (1.0 / ray_direction.z).abs() } else { f32::MAX };

        let closest_intersect: Option<Intersect> = None;

        // Traverse up to 250 blocks distance (Render Distance)
        for _ in 0..250 {
            if let Some(cube) = self.blocks.get(&current_voxel) {
                if let Some(intersect) = cube.ray_intersect(ray_origin, ray_direction) {
                    // DDA doesn't perfectly guarantee we hit the face entering the voxel if the ray 
                    // originates inside the block, but for our case, relying on Cube's intersect math 
                    // is robust enough to just return it directly.
                    return Some(intersect);
                }
            }

            // Move to the next voxel
            if t_max_x < t_max_y {
                if t_max_x < t_max_z {
                    current_voxel[0] += step_x;
                    t_max_x += t_delta_x;
                } else {
                    current_voxel[2] += step_z;
                    t_max_z += t_delta_z;
                }
            } else {
                if t_max_y < t_max_z {
                    current_voxel[1] += step_y;
                    t_max_y += t_delta_y;
                } else {
                    current_voxel[2] += step_z;
                    t_max_z += t_delta_z;
                }
            }
        }

        closest_intersect
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub struct World {
    pub grid: VoxelGrid,
    pub entities: Vec<Box<dyn RayIntersect>>,
}

impl World {
    pub fn new() -> Self {
        World {
            grid: VoxelGrid::new(),
            entities: Vec::new(),
        }
    }
    
    pub fn from_objects(raw_objects: Vec<Box<dyn RayIntersect>>) -> Self {
        let mut world = World::new();
        for obj in raw_objects {
            if let Some(cube) = obj.as_any().downcast_ref::<Cube>() {
                let x = (cube.min.x + 0.5).round() as i32;
                let y = (cube.min.y + 0.5).round() as i32;
                let z = (cube.min.z + 0.5).round() as i32;
                world.grid.insert(x, y, z, cube.clone());
            } else {
                world.entities.push(obj);
            }
        }
        world
    }
}

impl RayIntersect for World {
    fn ray_intersect(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Option<Intersect> {
        let mut closest = self.grid.ray_intersect(ray_origin, ray_direction);
        
        for entity in &self.entities {
            if let Some(intersect) = entity.ray_intersect(ray_origin, ray_direction) {
                if closest.as_ref().is_none_or(|c| intersect.distance < c.distance) {
                    closest = Some(intersect);
                }
            }
        }
        closest
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    
    fn update(&mut self, time: f32) {
        for entity in &mut self.entities {
            entity.update(time);
        }
    }
}
