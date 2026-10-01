use crate::color::Color;
use crate::texture::Texture;
use nalgebra_glm::Vec3;
use std::sync::Arc;

#[derive(Clone)]
pub struct Material {
    pub diffuse: Color,
    pub specular: f32,
    pub albedo: [f32; 4],
    pub refractive_index: f32,
    pub has_emission: bool,
    pub texture: Option<Arc<Texture>>,
    pub uv_scale: (f32, f32),
    pub uv_offset: (f32, f32),
    pub uv_rotated: bool,
    pub is_water: bool,
    pub is_portal: bool,
}

impl Material {
    pub fn new(diffuse: Color, specular: f32, albedo: [f32; 4]) -> Self {
        Material {
            diffuse,
            specular,
            albedo,
            refractive_index: 1.0,
            has_emission: false,
            texture: None,
            uv_scale: (1.0, 1.0),
            uv_offset: (0.0, 0.0),
            uv_rotated: false,
            is_water: false,
            is_portal: false,
        }
    }

    pub fn with_refractive_index(mut self, index: f32) -> Self {
        self.refractive_index = index;
        self
    }

    pub fn with_emission(mut self, has_emission: bool) -> Self {
        self.has_emission = has_emission;
        self
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

    pub fn with_uv_rotated(mut self, rotated: bool) -> Self {
        self.uv_rotated = rotated;
        self
    }

    pub fn with_water(mut self, is_water: bool) -> Self {
        self.is_water = is_water;
        self
    }

    pub fn with_portal(mut self, is_portal: bool) -> Self {
        self.is_portal = is_portal;
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

pub trait RayIntersect: Sync + Send {
    fn ray_intersect(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Option<Intersect>;
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
    fn update(&mut self, _time: f32) {}
}
