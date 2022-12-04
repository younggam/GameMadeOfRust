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
    pub fn octants(&self) -> Option<BVec3> {
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
        } else if t_min < 0. {
            Some(t_max)
        } else {
            Some(t_min)
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

    pub fn collision_system() {}

    ///Quick conversion from octant to leaf index.
    const fn octant_to_index(x: bool, y: bool, z: bool) -> usize {
        const STEP_X: usize = 4;
        const STEP_Y: usize = 2;
        const STEP_Z: usize = 1;
        STEP_X * x as usize + STEP_Y * y as usize + STEP_Z * z as usize
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
        let ret = if self.entities.len() < 4 && self.leaves.is_none() {
            self.entities.insert(entity)
        } else {
            if let None = self.leaves {
                //Directly insert to prevent recursive split.
                let ret = self.entities.insert(entity);
                self.split();
                ret
            } else {
                match &mut self.leaves {
                    Some(leaves) => match (entity.bound - self.bound.center()).octants() {
                        Some(BVec3 { x, y, z }) => {
                            leaves[Self::octant_to_index(x, y, z)].insert_inner(entity)
                        }
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
        let mut new_leaves = [
            Self::from_bound(self.bound.get_octant(false, false, false)),
            Self::from_bound(self.bound.get_octant(false, false, true)),
            Self::from_bound(self.bound.get_octant(false, true, false)),
            Self::from_bound(self.bound.get_octant(false, true, true)),
            Self::from_bound(self.bound.get_octant(true, false, false)),
            Self::from_bound(self.bound.get_octant(true, false, true)),
            Self::from_bound(self.bound.get_octant(true, true, false)),
            Self::from_bound(self.bound.get_octant(true, true, true)),
        ];
        self.entities.retain(
            |&entity| match (entity.bound - self.bound.center()).octants() {
                Some(BVec3 { x, y, z }) => {
                    let leaf = &mut new_leaves[Self::octant_to_index(x, y, z)];
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
        let ret = match (entity.bound - self.bound.center()).octants() {
            Some(BVec3 { x, y, z }) => {
                if let Some(ref mut leaves) = self.leaves {
                    match leaves[Self::octant_to_index(x, y, z)].remove_inner(entity) {
                        true => true,
                        false => self.entities.remove(&entity),
                    }
                } else {
                    self.entities.remove(&entity)
                }
            }
            None => self.entities.remove(&entity),
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
        match (bound - self.bound.center()).octants() {
            Some(BVec3 { x, y, z }) => {
                if let Some(ref leaves) = self.leaves {
                    leaves[Self::octant_to_index(x, y, z)].intersect(bound, f);
                }
            }
            _ => {}
        }
    }

    ///Return the bound and point raycast have hit first.
    pub fn raycast_hit(&self, ray: Ray, correction: f32) -> Option<(Entity, BoundingBox, Vec3)> {
        //Get hit point by just multiplying len between origin with dir.
        match self.raycast(ray) {
            Some((e, b, len)) => Some((e, b, ray.origin + ray.dir * (len - correction))),
            None => None,
        }
    }

    ///Return the bound raycast have hit first.
    pub fn raycast(&self, ray: Ray) -> Option<(Entity, BoundingBox, f32)> {
        //At least, ray should intersect nodes' bound
        if let Some(_) = self.bound.intersects_ray(ray) {
            let mut ret = None;
            //Checking all containing entities.
            for entity in self.entities.iter() {
                if let Some(len) = entity.bound.intersects_ray(ray) {
                    //result should be the shortest one.
                    ret = match ret {
                        Some((_, _, len2)) if len >= len2 => ret,
                        _ => Some((entity.entity, entity.bound, len)),
                    }
                }
            }
            //Also under leaves.
            if let Some(ref leaves) = self.leaves {
                for leaf in leaves.iter() {
                    if let Some((e, b, len)) = leaf.raycast(ray) {
                        ret = match ret {
                            Some((_, _, len2)) if len >= len2 => ret,
                            _ => Some((e, b, len)),
                        }
                    }
                }
            }
            ret
        } else {
            None
        }
    }
}
