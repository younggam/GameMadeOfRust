use crate::physics::ray::Ray;

use std::{
    cmp::Ordering,
    ops::{Add, MulAssign, Sub},
};

use bevy::{
    math::{BVec3, Quat, Vec3},
    prelude::Component,
};

///Aabb box. Min value must smaller than Max value in every axis.
#[derive(Component, Clone, Copy, PartialEq)]
pub struct AABB {
    min: Vec3,
    max: Vec3,
}

impl AABB {
    pub fn new(min: Vec3, max: Vec3) -> Self {
        if min.cmpge(max).any() || min.is_nan() || max.is_nan() {
            panic!(
                "min value of BoundingBox is greater than max or either min or max contains NaN"
            );
        }
        Self { min, max }
    }

    pub const unsafe fn new_unchecked(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    ///Determine min and max from size and zero offset.
    pub fn from_size(mut size: f32) -> Self {
        size = size.abs() * 0.5;
        Self::new(Vec3::splat(-size), Vec3::splat(size))
    }

    ///Determine min and max from size and offset.
    pub fn _from_size_offset(mut size: f32, offset: Vec3) -> Self {
        size = size.abs() * 0.5;
        Self::new(offset - size, offset + size)
    }

    //Extract aabb from shape vertices and objects' pos and rot.
    pub fn _from_points(points: &[Vec3], pos: Vec3, rot: Quat) -> Self {
        if points.len() < 3 {
            panic!("Number of points should be at least 3 to be polygon.");
        } else {
            let mut min = Vec3::splat(f32::INFINITY);
            let mut max = Vec3::splat(f32::NEG_INFINITY);
            for point in points {
                let point = rot.mul_vec3(*point + pos);
                min = min.min(point);
                max = max.max(point);
            }
            Self::new(min, max)
        }
    }

    pub fn min(&self) -> Vec3 {
        self.min
    }

    pub fn max(&self) -> Vec3 {
        self.max
    }

    pub fn length(&self) -> Vec3 {
        self.max - self.min
    }

    pub fn _x_length(&self) -> f32 {
        self.max.x - self.min.x
    }

    pub fn _y_length(&self) -> f32 {
        self.max.y - self.min.y
    }

    pub fn _z_length(&self) -> f32 {
        self.max.z - self.min.z
    }

    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    pub fn center_x(&self) -> f32 {
        (self.min.x + self.max.x) * 0.5
    }

    pub fn center_y(&self) -> f32 {
        (self.min.y + self.max.y) * 0.5
    }

    pub fn center_z(&self) -> f32 {
        (self.min.z + self.max.z) * 0.5
    }

    ///Extends bounding box exponentially until size is bigger than other.
    pub fn extend_for(mut self, other: &Self, mut f: impl FnMut(AABB)) {
        while self.min.x > other.min.x || self.min.y > other.min.y || self.min.z > other.min.z {
            self.min -= self.length();
            f(self);
        }
        while self.max.x < other.max.x || self.max.y < other.max.y || self.max.z < other.max.z {
            self.max += self.length();
            f(self);
        }
    }

    ///Determines which octant from origin this box is placed. True is positive, false is negative.
    pub fn octant(&self) -> Option<BVec3> {
        let x_p = self.min.x >= 0. && self.max.x > 0.;
        let x_n = self.min.x < 0. && self.max.x <= 0.;
        let y_p = self.min.y >= 0. && self.max.y > 0.;
        let y_n = self.min.y < 0. && self.max.y <= 0.;
        let z_p = self.min.z >= 0. && self.max.z > 0.;
        let z_n = self.min.z < 0. && self.max.z <= 0.;

        if x_p ^ x_n && y_p ^ y_n && z_p ^ z_n {
            Some(BVec3::new(x_p, y_p, z_p))
        } else {
            None
        }
    }

    ///Get octant of this box's center as origin.
    pub fn get_octant(&self, bvec3: BVec3) -> Self {
        let (min_x, max_x) = if bvec3.x {
            (self.center_x(), self.max.x)
        } else {
            (self.min.x, self.center_x())
        };
        let (min_y, max_y) = if bvec3.y {
            (self.center_y(), self.max.y)
        } else {
            (self.min.y, self.center_y())
        };
        let (min_z, max_z) = if bvec3.z {
            (self.center_z(), self.max.z)
        } else {
            (self.min.z, self.center_z())
        };
        Self::new(
            Vec3::new(min_x, min_y, min_z),
            Vec3::new(max_x, max_y, max_z),
        )
    }

    ///Check where point is lying on in coordinate system that origin is center of bound.
    /// - `Ordering::Greater` if the element is positive.
    /// - `Ordering::Less` if the element is negative.
    /// - `Ordering::Equal` if the element cannot be distinguished with 0.
    pub fn is_on_octant(&self, point: Vec3) -> [Ordering; 3] {
        let center = self.center();
        [
            if (point.x - center.x).abs() <= f32::EPSILON {
                Ordering::Equal
            } else if point.x < center.x {
                Ordering::Less
            } else {
                Ordering::Greater
            },
            if (point.y - center.y).abs() <= f32::EPSILON {
                Ordering::Equal
            } else if point.y < center.y {
                Ordering::Less
            } else {
                Ordering::Greater
            },
            if (point.z - center.z).abs() <= f32::EPSILON {
                Ordering::Equal
            } else if point.z < center.z {
                Ordering::Less
            } else {
                Ordering::Greater
            },
        ]
    }

    ///Checks whether this and other bounding box intersected. Exclusive bound line.
    pub fn _intersects(&self, other: &Self) -> bool {
        self.min.cmplt(other.max).all() && self.max.cmpgt(other.min).all()
    }

    ///Checks whether point is in bounding box.
    pub fn _overlaps_point(&self, point: Vec3) -> bool {
        self.min.cmplt(point).all() && self.max.cmplt(point).all()
    }

    ///Checks if ray is penetrating box.
    pub fn intersects_ray(&self, ray: &Ray) -> Option<f32> {
        self.intersects_ray_raw(ray)
            .map(|(t_min, t_max)| if t_min <= 0. { t_max } else { t_min })
    }

    ///Checks if ray is penetrating box and returns raw data for two location on bound that ray passes.
    pub fn intersects_ray_raw(&self, ray: &Ray) -> Option<(f32, f32)> {
        let mut t_min = f32::NEG_INFINITY;
        let mut t_max = f32::INFINITY;

        let d_min = ray.t(self.min);
        let d_max = ray.t(self.max);
        for i in 0..3 {
            t_min = t_min.max(d_min[i].min(d_max[i]).min(t_max));
            t_max = t_max.min(d_min[i].max(d_max[i]).max(t_min));
        }

        if t_max <= 0. || t_min >= t_max {
            None
        } else {
            Some((t_min, t_max))
        }
    }
}

impl Add<f32> for AABB {
    type Output = AABB;

    fn add(self, rhs: f32) -> Self::Output {
        Self::Output {
            min: self.min + rhs,
            max: self.max + rhs,
        }
    }
}

impl Sub<f32> for AABB {
    type Output = AABB;

    fn sub(self, rhs: f32) -> Self::Output {
        Self::Output {
            min: self.min - rhs,
            max: self.max - rhs,
        }
    }
}

impl MulAssign<f32> for AABB {
    fn mul_assign(&mut self, rhs: f32) {
        self.min *= rhs;
        self.max *= rhs;
    }
}

impl Add<Vec3> for AABB {
    type Output = AABB;

    fn add(self, rhs: Vec3) -> Self::Output {
        Self::Output {
            min: self.min + rhs,
            max: self.max + rhs,
        }
    }
}

impl Sub<Vec3> for AABB {
    type Output = AABB;

    fn sub(self, rhs: Vec3) -> Self::Output {
        Self::Output {
            min: self.min - rhs,
            max: self.max - rhs,
        }
    }
}
