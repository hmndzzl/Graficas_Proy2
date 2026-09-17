use crate::color::Color;
use crate::texture::Texture;
use nalgebra_glm::Vec3;
use std::sync::Arc;

#[derive(Clone)]
pub struct Material {
    pub diffuse: Color,
    pub specular: f32,
    pub albedo: [f32; 3],
    pub texture: Option<Arc<Texture>>,
    pub uv_scale: (f32, f32),
    pub uv_offset: (f32, f32),
}

impl Material {
    pub fn new(diffuse: Color, specular: f32, albedo: [f32; 3]) -> Self {
        Material {
            diffuse,
            specular,
            albedo,
            texture: None,
            uv_scale: (1.0, 1.0),
            uv_offset: (0.0, 0.0),
        }
    }

    pub fn with_texture(mut self, texture: Arc<Texture>) -> Self {
        self.texture = Some(texture);
        self
    }

    pub fn with_uv(mut self, scale: (f32, f32), offset: (f32, f32)) -> Self {
        self.uv_scale = scale;
        self.uv_offset = offset;
        self
    }
}

#[derive(Clone)]
pub struct Intersect {
    pub point: Vec3,
    pub normal: Vec3,
    pub distance: f32,
    pub material: Material,
    pub u: f32,
    pub v: f32,
}

pub trait RayIntersect {
    fn ray_intersect(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Option<Intersect>;
}
