use bevy::prelude::Component;

pub mod aabb;
pub mod ray;
pub mod octree;

///Currently marks whether entity could be collide.
#[derive(Component)]
pub struct Collides;
