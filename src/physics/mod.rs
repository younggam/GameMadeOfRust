use std::{
    cmp::Ordering,
    collections::BTreeSet,
    ops::{Add, Deref, Sub},
};

use bevy::prelude::*;

///Currently marks whether entity could be collide.
#[derive(Component)]
pub struct Collides;

///Aabb box. Min value must smaller or equal than Max value in every axis.
#[derive(Component, Clone, Copy)]
pub struct BoundingBox {
    min: Vec3,
    max: Vec3,
}

impl BoundingBox {
    pub fn new(min: Vec3, max: Vec3) -> Self {
        if min.cmpgt(max).any() {
            panic!("min value of BoundingBox is greater than max");
        }
        Self { min, max }
    }

    ///Determine min and max from size and zero offset.
    pub fn from_size(size: f32) -> Self {
        Self {
            min: Vec3::splat(-size * 0.5),
            max: Vec3::splat(size * 0.5),
        }
    }

    ///Determine min and max from size and offset.
    pub fn from_size_offset(size: f32, offset: Vec3) -> Self {
        Self {
            min: offset - size * 0.5,
            max: offset + size * 0.5,
        }
    }

    pub fn from_points(points: &[Vec3]) -> Option<Self> {
        //점이 최소 3개는 되어야 다각형 이다.
        if points.len() < 3 {
            None
        } else {
            let mut min = Vec3::splat(f32::INFINITY);
            let mut max = Vec3::splat(f32::NEG_INFINITY);
            for point in points {
                min = min.min(*point);
                max = max.max(*point);
            }
            Some(BoundingBox { min, max })
        }
    }

    pub fn length(&self) -> Vec3 {
        self.max - self.min
    }

    pub fn x_length(&self) -> f32 {
        self.max.x - self.min.x
    }

    pub fn y_length(&self) -> f32 {
        self.max.y - self.min.y
    }

    pub fn z_length(&self) -> f32 {
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
    pub fn get_octant(&self, x: bool, y: bool, z: bool) -> Self {
        let (min_x, max_x) = if x {
            (self.center_x(), self.max.x)
        } else {
            (self.min.x, self.center_x())
        };
        let (min_y, max_y) = if y {
            (self.center_y(), self.max.y)
        } else {
            (self.min.y, self.center_y())
        };
        let (min_z, max_z) = if z {
            (self.center_z(), self.max.z)
        } else {
            (self.min.z, self.center_z())
        };
        Self::new(
            Vec3::new(min_x, min_y, min_z),
            Vec3::new(max_x, max_y, max_z),
        )
    }

    pub fn get_octants(&self) -> [Self; 8] {
        let BoundingBox { min, max } = *self;
        let center = self.center();
        [
            Self {
                min: min,
                max: center,
            },
            Self {
                min: Vec3::new(min.x, min.y, center.z),
                max: Vec3::new(center.x, center.y, max.z),
            },
            Self {
                min: Vec3::new(min.x, center.y, min.z),
                max: Vec3::new(center.x, max.y, center.z),
            },
            Self {
                min: Vec3::new(min.x, center.y, center.z),
                max: Vec3::new(center.x, max.y, max.z),
            },
            Self {
                min: Vec3::new(center.x, min.y, min.z),
                max: Vec3::new(max.x, center.y, center.z),
            },
            Self {
                min: Vec3::new(center.x, min.y, center.z),
                max: Vec3::new(max.x, center.y, max.z),
            },
            Self {
                min: Vec3::new(center.x, center.y, min.z),
                max: Vec3::new(max.x, max.y, center.z),
            },
            Self {
                min: center,
                max: self.max,
            },
        ]
    }

    ///Check where point is lying on in coordinate system that origin is center of bound.
    /// - 1 if the element is positive.
    /// - -1 if the element is negative.
    /// - 0 if the element cannot be distinguished with 0.
    pub fn is_on_octant(&self, point: Vec3) -> IVec3 {
        let center = self.center();

        IVec3::new(
            if (point.x - center.x).abs() <= f32::EPSILON {
                0
            } else if point.x < center.x {
                -1
            } else {
                1
            },
            if (point.y - center.y).abs() <= f32::EPSILON {
                0
            } else if point.y < center.y {
                -1
            } else {
                1
            },
            if (point.z - center.z).abs() <= f32::EPSILON {
                0
            } else if point.z < center.z {
                -1
            } else {
                1
            },
        )
    }

    ///Checks whether this and other bounding box intersected.
    pub fn intersects(&self, other: &Self) -> bool {
        self.min.cmplt(other.max).all() && self.max.cmpgt(other.min).all()
    }

    ///Checks whether point is in bounding box.
    pub fn overlaps_point(&self, point: Vec3) -> bool {
        self.min.cmplt(point).all() && self.max.cmplt(point).all()
    }

    ///Checks if ray is penetrating box.
    pub fn intersects_ray(&self, ray: Ray) -> Option<f32> {
        self.intersects_ray_raw(ray)
            .map(|(t_min, t_max)| if t_min <= 0. { t_max } else { t_min })
    }

    ///Checks if ray is penetrating box and returns raw data for two location on bound that ray passes.
    pub fn intersects_ray_raw(&self, ray: Ray) -> Option<(f32, f32)> {
        let mut t_min = f32::NEG_INFINITY;
        let mut t_max = f32::INFINITY;

        for i in 0..3 {
            let d_min = (self.min[i] - ray.origin[i]) * ray.recip_dir[i];
            let d_max = (self.max[i] - ray.origin[i]) * ray.recip_dir[i];

            t_min = t_min.max(d_min.min(d_max).min(t_max));
            t_max = t_max.min(d_min.max(d_max).max(t_min));
        }

        if t_max <= 0. || t_min >= t_max {
            None
        } else {
            Some((t_min, t_max))
        }
    }
}

impl Add<f32> for BoundingBox {
    type Output = BoundingBox;

    fn add(self, rhs: f32) -> Self::Output {
        Self::Output {
            min: self.min + rhs,
            max: self.max + rhs,
        }
    }
}

impl Sub<f32> for BoundingBox {
    type Output = BoundingBox;

    fn sub(self, rhs: f32) -> Self::Output {
        Self::Output {
            min: self.min - rhs,
            max: self.max - rhs,
        }
    }
}

impl Add<Vec3> for BoundingBox {
    type Output = BoundingBox;

    fn add(self, rhs: Vec3) -> Self::Output {
        Self::Output {
            min: self.min + rhs,
            max: self.max + rhs,
        }
    }
}

impl Sub<Vec3> for BoundingBox {
    type Output = BoundingBox;

    fn sub(self, rhs: Vec3) -> Self::Output {
        Self::Output {
            min: self.min - rhs,
            max: self.max - rhs,
        }
    }
}

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

    ///Extract octant from ray's initial traverse at certain spot.
    /// - None if ray is included on axis and base planes.
    pub fn octant_at(&self, pivot: f32, bound: BoundingBox) -> Option<BVec3> {
        let mut octant = bound.is_on_octant(self.origin + self.dir * pivot);
        if octant.x == 0 && self.dir.x != 0. {
            if self.dir.x > 0. {
                octant.x = 1
            } else if self.dir.x < 0. {
                octant.x = -1
            }
        }
        if octant.y == 0 && self.dir.y != 0. {
            if self.dir.y > 0. {
                octant.y = 1
            } else if self.dir.y < 0. {
                octant.y = -1
            }
        }
        if octant.z == 0 && self.dir.z != 0. {
            if self.dir.z > 0. {
                octant.z = 1
            } else if self.dir.z < 0. {
                octant.z = -1
            }
        }
        if octant.cmpeq(IVec3::ZERO).any() {
            None
        } else {
            Some(octant.cmpgt(IVec3::ZERO))
        }
    }

    ///Get next octant from point, where ray is touching on previous octant.
    ///Ray pivot should lie on previous octant's surface for accurate result.
    pub fn next_octant(&self, mut octant: BVec3, pivot: f32, bound: BoundingBox) -> BVec3 {
        let check = self.origin + pivot * self.dir
            - bound.get_octant(octant.x, octant.y, octant.z).center();
        let check_abs = check.abs();
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

///Container for entity and bounding box for octree.
#[derive(Copy, Clone)]
pub struct OctreeEntity {
    entity: Entity,
    bound: BoundingBox,
}

impl OctreeEntity {
    pub fn new(entity: Entity, bound: BoundingBox) -> Self {
        Self { entity, bound }
    }
}

impl Deref for OctreeEntity {
    type Target = BoundingBox;

    fn deref(&self) -> &Self::Target {
        &self.bound
    }
}

impl Eq for OctreeEntity {}

impl PartialEq for OctreeEntity {
    fn eq(&self, other: &Self) -> bool {
        self.entity.eq(&other.entity)
    }
}

impl PartialOrd for OctreeEntity {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.entity.partial_cmp(&other.entity)
    }
}

impl Ord for OctreeEntity {
    fn cmp(&self, other: &Self) -> Ordering {
        self.entity.cmp(&other.entity)
    }
}

///Octree that 3 dimensional version of binary heap. Useful for broad collision via aabb.
#[derive(Component)]
pub struct OctreeNode {
    ///Bound of itself.
    pub bound: BoundingBox,
    ///Entities that a few or doesn't fit with leaves.
    entities: BTreeSet<OctreeEntity>,
    ///Leaf nodes that divides space.
    leaves: Option<Box<[OctreeNode; 8]>>,
    ///Total amounts of entities below.
    length: usize,
}

impl OctreeNode {
    const SPLIT_THRESHOLD: usize = 7;

    pub fn new(size: f32, offset: Vec3) -> OctreeNode {
        Self::from_bound(BoundingBox::from_size_offset(size, offset))
    }

    pub fn from_bound(bound: BoundingBox) -> Self {
        OctreeNode {
            bound,
            entities: BTreeSet::new(),
            leaves: None,
            length: 0,
        }
    }

    // pub fn collision_system(&self) {
    //     let mut i = self.entities.iter();
    //     while let Some(entity0) = i.next() {
    //         let mut j = i.clone();
    //         for entity1 in j {
    //             if entity0.intersects(entity1) && entity0.collides(entity1) {
    //                 //do something
    //             }
    //         }
    //         if let Some(leaves) = &self.leaves {
    //             for leaf in leaves {
    //                 leaf.collision_with(entity0);
    //             }
    //         }
    //     }
    //     if let Some(leaves) = &self.leaves {
    //         for leaf in leaves {
    //             leaf.collision_system();
    //         }
    //     }
    // }
    //
    // pub fn collision_with(&self, entity0: OctreeEntity) {
    //     if entity0.intersects(&self.bound) {
    //         for entity1 in self.entities.iter() {
    //             if entity0.intersects(entity1) && entity0.collides(entity1) {
    //                 //do something.
    //             }
    //         }
    //         if let Some(leaves) = &self.leaves {
    //             for leaf in leaves {
    //                 leaf.collision_with(entity0);
    //             }
    //         }
    //     }
    // }

    ///Quick conversion from octant to leaf index.
    const fn octant_to_index(octant: BVec3) -> usize {
        const STEP_X: usize = 4;
        const STEP_Y: usize = 2;
        const STEP_Z: usize = 1;
        STEP_X * octant.x as usize + STEP_Y * octant.y as usize + STEP_Z * octant.z as usize
    }

    pub fn len(&self) -> usize {
        self.length
    }

    ///If node and its leaves entirely empty.
    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

    ///Return is whether entity doesn't already exist.
    pub fn insert(&mut self, entity: Entity, bound: BoundingBox) -> bool {
        let ret = self.insert_inner(OctreeEntity::new(entity, bound));
        println!("counts {}", self.len());
        ret
    }

    fn insert_inner(&mut self, entity: OctreeEntity) -> bool {
        //Kinda threshold to prevent frequent division.
        let ret = if self.entities.len() < Self::SPLIT_THRESHOLD && self.leaves.is_none() {
            self.entities.insert(entity)
        } else {
            if let None = self.leaves {
                //Directly insert to prevent recursive split.
                let ret = self.entities.insert(entity);
                self.split();
                ret
            } else {
                match &mut self.leaves {
                    Some(leaves) => match (entity.bound - self.bound.center()).octant() {
                        Some(octant) => leaves[Self::octant_to_index(octant)].insert_inner(entity),
                        None => self.entities.insert(entity),
                    },
                    _ => false,
                }
            }
        };
        if ret {
            self.length += 1;
        }
        ret
    }

    //Split existing entities.
    fn split(&mut self) {
        println!("split");
        let mut new_leaves = self.bound.get_octants().map(|b| Self::from_bound(b));
        self.entities.retain(
            |&entity| match (entity.bound - self.bound.center()).octant() {
                Some(octant) => {
                    let leaf = &mut new_leaves[Self::octant_to_index(octant)];
                    if leaf.entities.insert(entity) {
                        leaf.length += 1;
                    }
                    false
                }
                None => true,
            },
        );
        self.leaves = Some(Box::new(new_leaves));
    }

    ///Return is whether existed entity is removed.
    pub fn remove(&mut self, entity: Entity, bound: BoundingBox) -> bool {
        let ret = self.remove_inner(OctreeEntity::new(entity, bound));
        println!("counts {}", self.len());
        ret
    }

    fn remove_inner(&mut self, entity: OctreeEntity) -> bool {
        let ret = if let Some(ref mut leaves) = self.leaves {
            match (entity.bound - self.bound.center()).octant() {
                Some(octant) => match leaves[Self::octant_to_index(octant)].remove_inner(entity) {
                    true => true,
                    false => self.entities.remove(&entity),
                },
                None => self.entities.remove(&entity),
            }
        } else {
            self.entities.remove(&entity)
        };
        if ret {
            self.length -= 1;
            if self.is_empty() {
                println!("unsplit");
                self.leaves = None;
            }
        }
        ret
    }

    ///Iterating entities that intersects with given bounding box.
    pub fn intersect(&self, bound: BoundingBox, f: impl Fn(&Entity)) {
        for entity in self.entities.iter() {
            if entity.bound.intersects(&bound) {
                f(&entity.entity);
            }
        }
        match (bound - self.bound.center()).octant() {
            Some(octant) => {
                if let Some(ref leaves) = self.leaves {
                    leaves[Self::octant_to_index(octant)].intersect(bound, f);
                }
            }
            _ => {}
        }
    }

    // ///Return the bound and point raycast have hit first.
    // pub fn raycast_hit(&self, ray: Ray, correction: f32) -> Option<(Entity, BoundingBox, Vec3)> {
    //     //Get hit point by just multiplying len between origin with dir.
    //     match self.raycast(ray) {
    //         Some((e, b, len)) => Some((e, b, ray.origin + ray.dir * (len - correction))),
    //         None => None,
    //     }
    // }
    //
    // ///Return the bound raycast have hit first.
    // pub fn raycast(&self, ray: Ray) -> Option<(Entity, BoundingBox, f32)> {
    //     //At least, ray should intersect nodes' bound
    //     if let Some(_) = self.bound.intersects_ray(ray) {
    //         let mut ret = None;
    //         //Checking all containing entities.
    //         for entity in self.entities.iter() {
    //             if let Some(candidate) = entity.bound.intersects_ray(ray) {
    //                 //result should be the shortest one.
    //                 ret = match ret {
    //                     Some((_, _, len)) if candidate >= len => ret,
    //                     _ => Some((entity.entity, entity.bound, candidate)),
    //                 }
    //             }
    //         }
    //         //Also under leaves.
    //         if let Some(ref leaves) = self.leaves {
    //             for leaf in leaves.iter() {
    //                 if let Some((e, b, candidate)) = leaf.raycast(ray) {
    //                     ret = match ret {
    //                         Some((_, _, len)) if candidate >= len => ret,
    //                         _ => Some((e, b, candidate)),
    //                     }
    //                 }
    //             }
    //         }
    //         ret
    //     } else {
    //         None
    //     }
    // }

    pub fn raycast_hit(&self, ray: Ray, correction: f32) -> Option<(Entity, BoundingBox, Vec3)> {
        let mut len = f32::INFINITY;
        match self.raycast_inner(ray, &mut len, &mut 0.) {
            Some((e, b)) => Some((e, b, ray.origin + ray.dir * (len - correction))),
            None => None,
        }
    }

    pub fn raycast(&self, ray: Ray) -> Option<(Entity, BoundingBox, f32)> {
        let mut len = f32::INFINITY;
        match self.raycast_inner(ray, &mut len, &mut 0.) {
            Some((e, b)) => Some((e, b, len)),
            None => None,
        }
    }

    ///Return the bound raycast have hit first.
    pub fn raycast_inner(
        &self,
        ray: Ray,
        len: &mut f32,
        pivot: &mut f32,
    ) -> Option<(Entity, BoundingBox)> {
        //At least, ray should intersect node's bound
        if let Some((_, t_max)) = self.bound.intersects_ray_raw(ray) {
            let mut ret = None;
            //Checking all containing entities.
            for entity in self.entities.iter() {
                if let Some(candidate) = entity.bound.intersects_ray(ray) {
                    //result should be the shortest one.
                    if candidate < *len {
                        ret = Some((entity.entity, entity.bound));
                        *len = candidate;
                    }
                }
            }

            //Checking leaves.
            if let Some(ref leaves) = self.leaves {
                let prev_pivot = *pivot;
                //Determine octant.
                match ray.octant_at(*pivot, self.bound) {
                    Some(mut octant) => loop {
                        //Get result of raycast on leaf.
                        match leaves[Self::octant_to_index(octant)].raycast_inner(ray, len, pivot) {
                            //First success is if and only if the shortest raycast on the leaves.
                            tmp @ Some(_) => {
                                ret = tmp;
                                break;
                            }
                            //Fail then shift leaf.
                            None => {
                                //Covers case that ray doesn't intersect even bound of leaf.
                                if *pivot == prev_pivot {
                                    *pivot = t_max
                                };
                                let prev_octant = octant;
                                octant = ray.next_octant(octant, *pivot, self.bound);
                                //Dead end of ray through leaves.
                                if octant == prev_octant {
                                    break;
                                }
                            }
                        }
                    },
                    //Discard if ray is lie on xy, yz, xz plane and x, y, z axis
                    None => {}
                }
            }

            //Shift pivot if there is no result.
            if let None = ret {
                *pivot = t_max;
            }
            ret
        } else {
            None
        }
    }
}
