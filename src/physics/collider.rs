use crate::physics::aabb::AABB;
use bevy::prelude::*;

#[derive(Component, Clone)]
pub struct Collider {
    shape: Shape,
}

impl Collider {
    pub fn from_shape(shape: Shape) -> Self {
        Self { shape }
    }

    pub fn aabb(&self, transform: &Transform) -> AABB {
        self.shape.aabb(transform)
    }

    pub fn shape(&self) -> Shape {
        self.shape.clone()
    }
}

#[derive(Clone)]
pub enum Shape {
    Sphere {
        radius: f32,
    },
    ///Sphere that is cut. *Note* Shape below are only for blueprint.
    CutSphere {
        radius: f32,
        cut: f32,
    },
}

impl Shape {
    pub fn aabb(&self, transform: &Transform) -> AABB {
        match self {
            Shape::Sphere { radius } => sphere_aabb(*radius, transform),
            Shape::CutSphere { radius, cut } => cut_sphere_aabb(*radius, *cut, transform),
        }
    }
}

fn sphere_aabb(radius: f32, transform: &Transform) -> AABB {
    AABB::from_size_offset(radius * 2., transform.translation)
}

fn cut_sphere_aabb(radius: f32, cut: f32, transform: &Transform) -> AABB {
    AABB::from_points(&[
        transform.transform_point(Vec3::new(radius, 0., 0.)),
        transform.transform_point(Vec3::new(-radius, 0., 0.)),
        transform.transform_point(Vec3::new(0., radius, 0.)),
        transform.transform_point(Vec3::new(0., -cut, 0.)),
        transform.transform_point(Vec3::new(0., 0., radius)),
        transform.transform_point(Vec3::new(0., 0., -radius)),
    ])
}
