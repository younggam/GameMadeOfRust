use crate::physics::aabb::AABB;

use std::cmp::Ordering;

use bevy::{
    math::{BVec3, Vec3},
    prelude::Entity,
};

///Caching ray data.
#[derive(Copy, Clone)]
pub struct Ray {
    origin: Vec3,
    dir: Vec3,
    recip_dir: Vec3,
}

impl Ray {
    pub fn new(origin: Vec3, dir: Vec3) -> Self {
        Self {
            origin,
            dir,
            recip_dir: dir.recip(),
        }
    }

    pub fn point(&self, t: f32) -> Vec3 {
        self.origin + self.dir * t
    }

    pub fn t(&self, vec3: Vec3) -> Vec3 {
        (vec3 - self.origin) * self.recip_dir
    }

    ///Extract octant from ray's initial traverse at certain spot.
    /// - None if ray is included on axis and base planes.
    pub fn octant_at(&self, pivot: f32, aabb: AABB) -> Option<BVec3> {
        let [mut x, mut y, mut z] = aabb.is_on_octant(self.origin + self.dir * pivot);
        if x.is_eq() && self.dir.x != 0. {
            if self.dir.x > 0. {
                x = Ordering::Greater
            } else if self.dir.x < 0. {
                x = Ordering::Less
            }
        }
        if y.is_eq() && self.dir.y != 0. {
            if self.dir.y > 0. {
                y = Ordering::Greater
            } else if self.dir.y < 0. {
                y = Ordering::Less
            }
        }
        if z.is_eq() && self.dir.z != 0. {
            if self.dir.z > 0. {
                z = Ordering::Greater
            } else if self.dir.z < 0. {
                z = Ordering::Less
            }
        }
        if x.is_eq() || y.is_eq() || z.is_eq() {
            None
        } else {
            Some(BVec3::new(x.is_gt(), y.is_gt(), z.is_gt()))
        }
    }

    ///Get next octant from point, where ray is touching on previous octant.
    ///Ray pivot should lie on previous octant's surface for accurate result.
    pub fn next_octant(&self, mut octant: BVec3, pivot: f32, bound: AABB) -> BVec3 {
        //Get vector from given octant's center to ray pivot position.
        let check = self.origin + pivot * self.dir - bound.get_octant(octant).center();
        //For determine which element of vector is the greatest.
        let check_abs = check.abs();
        //Determining which element is the greatest. It can be more than one.
        let delta = if check_abs.x > check_abs.y {
            if check_abs.x > check_abs.z {
                BVec3::new(true, false, false)
            } else if check_abs.z > check_abs.x {
                BVec3::new(false, false, true)
            } else {
                BVec3::new(true, false, true)
            }
        } else if check_abs.y > check_abs.x {
            if check_abs.y > check_abs.z {
                BVec3::new(false, true, false)
            } else if check_abs.z > check_abs.y {
                BVec3::new(false, false, true)
            } else {
                BVec3::new(false, true, true)
            }
        } else {
            if check_abs.x > check_abs.z {
                BVec3::new(true, true, false)
            } else if check_abs.z > check_abs.x {
                BVec3::new(false, false, true)
            } else {
                BVec3::new(true, true, true)
            }
        };

        //If certain axis value is the one or tied greatest, shift octant to that direction.
        if delta.x {
            octant.x = check.x > 0.;
        }
        if delta.y {
            octant.y = check.y > 0.;
        }
        if delta.z {
            octant.z = check.z > 0.;
        }

        octant
    }
}

pub struct RayHitInfo {
    pub entity: Entity,
    pub aabb: AABB,
    ///Distance
    pub t: f32,
}

impl RayHitInfo {
    pub fn new(entity: Entity, aabb: AABB, t: f32) -> Self {
        Self {
            entity,
            aabb,
            t,
        }
    }
}
