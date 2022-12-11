use std::{
    cmp::Ordering,
    collections::BTreeSet,
    ops::{Add, Deref, MulAssign, Sub},
};

use bevy::prelude::*;

///Currently marks whether entity could be collide.
#[derive(Component)]
pub struct Collides;

///Aabb box. Min value must smaller than Max value in every axis.
#[derive(Component, Clone, Copy, PartialEq)]
pub struct AABB {
    min: Vec3,
    max: Vec3,
}

#[allow(dead_code)]
impl AABB {
    pub fn new(min: Vec3, max: Vec3) -> Self {
        if min.cmpge(max).any() || min.is_nan() || max.is_nan() {
            panic!(
                "min value of BoundingBox is greater than max or either min or max contains NaN"
            );
        }
        Self { min, max }
    }

    ///Determine min and max from size and zero offset.
    pub fn from_size(mut size: f32) -> Self {
        size = size.abs();
        Self::new(Vec3::splat(-size * 0.5), Vec3::splat(size * 0.5))
    }

    ///Determine min and max from size and offset.
    pub fn from_size_offset(mut size: f32, offset: Vec3) -> Self {
        size = size.abs();
        Self::new(offset - size * 0.5, offset + size * 0.5)
    }

    //Extract aabb from shape vertices and objects' pos and rot.
    pub fn from_points(points: &[Vec3], pos: Vec3, rot: Quat) -> Self {
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
        let Self { min, max } = *self;
        let center = self.center();
        [
            Self::new(min, center),
            Self::new(
                Vec3::new(min.x, min.y, center.z),
                Vec3::new(center.x, center.y, max.z),
            ),
            Self::new(
                Vec3::new(min.x, center.y, min.z),
                Vec3::new(center.x, max.y, center.z),
            ),
            Self::new(
                Vec3::new(min.x, center.y, center.z),
                Vec3::new(center.x, max.y, max.z),
            ),
            Self::new(
                Vec3::new(center.x, min.y, min.z),
                Vec3::new(max.x, center.y, center.z),
            ),
            Self::new(
                Vec3::new(center.x, min.y, center.z),
                Vec3::new(max.x, center.y, max.z),
            ),
            Self::new(
                Vec3::new(center.x, center.y, min.z),
                Vec3::new(max.x, max.y, center.z),
            ),
            Self::new(center, max),
        ]
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
    pub fn octant_at(&self, pivot: f32, bound: AABB) -> Option<BVec3> {
        let [mut x, mut y, mut z] = bound.is_on_octant(self.origin + self.dir * pivot);
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
    aabb: AABB,
}

impl OctreeEntity {
    pub fn new(entity: Entity, bound: AABB) -> Self {
        Self {
            entity,
            aabb: bound,
        }
    }
}

impl Deref for OctreeEntity {
    type Target = AABB;

    fn deref(&self) -> &Self::Target {
        &self.aabb
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

#[derive(Component)]
pub struct Octree {
    ///Index of root node from pool.
    root: usize,
    ///Base aabb for creating root node.
    base_aabb: AABB,
    ///Kinda node pool
    nodes: Vec<OctreeNodeA>,
    node_count: usize,
    node_capacity: usize,
    ///Index of idle root node from pool.
    idle: usize,
    len: usize,
}

impl Octree {
    const NULL_INDEX: usize = usize::MAX;
    const SPLIT_THRESHOLD: usize = 4;

    pub fn new(size: f32, offset: Vec3) -> Self {
        Self::from_aabb(AABB::from_size_offset(size, offset))
    }

    pub fn from_aabb(aabb: AABB) -> Self {
        let node_capacity = 16;
        Self {
            root: Self::NULL_INDEX,
            base_aabb: aabb,
            nodes: Vec::with_capacity(node_capacity),
            node_count: 0,
            node_capacity,
            idle: Self::NULL_INDEX,
            len: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    ///If node and its leaves entirely empty.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    ///Create a node or find and set a idle node.
    fn get_or_create_node(&mut self, aabb: AABB, parent: usize) -> usize {
        if self.idle == Self::NULL_INDEX {
            //Create a node if there is no idle node.
            self.nodes.push(OctreeNodeA::new(aabb, parent));
            return self.nodes.len() - 1;
        }
        //Get and set idle node.
        let index = self.idle;
        let node = &mut self.nodes[self.idle];
        self.idle = node.parent;
        node.aabb = aabb;
        node.parent = parent;
        index
    }

    /// Create a node or find a idle node.
    fn idles_node(&mut self, index: usize, octant_index: usize) {
        let parent_index = self.nodes[index].parent;
        if parent_index != Self::NULL_INDEX {
            //Remove children from parent.
            let parent = &mut self.nodes[parent_index];
            parent.children[octant_index] = Self::NULL_INDEX;
            parent.children_len -= 1;
        } else {
            //No nodes left.
            self.root = Self::NULL_INDEX;
        }
        self.nodes[index].parent = self.idle;
        self.idle = index;
    }

    ///Return is whether entity doesn't already exist.
    pub fn insert(&mut self, entity: Entity, aabb: AABB) -> bool {
        let entity = OctreeEntity::new(entity, aabb);
        let mut index = self.root;
        if index == Self::NULL_INDEX {
            //When there is no node in tree at all.
            index = self.get_or_create_node(self.base_aabb, Self::NULL_INDEX);
        }
        let mut ret = false;
        loop {
            let node = &mut self.nodes[index];
            //Kinda threshold to prevent frequent division.
            if node.entities.len() < Self::SPLIT_THRESHOLD && node.children_len == 0 {
                ret = node.entities.insert(entity);
                break;
            } else {
                //Whether entity is fit in node's arbitrary octant.
                match (entity.aabb - node.aabb.center()).octant() {
                    Some(octant) => {
                        //Determine octant and put entity in the child
                        let octant_index = OctreeNodeA::octant_to_index(octant);
                        let child_index = node.children[octant_index];
                        if child_index == Self::NULL_INDEX {
                            //When child is not created yet, create and set.
                            let new_aabb = node.aabb.get_octant(octant.x, octant.y, octant.z);
                            let parent_index = index;
                            index = self.get_or_create_node(new_aabb, index);
                            self.nodes[parent_index].children[octant_index] = index;
                            break;
                        } else {
                            index = child_index
                        }
                    }
                    None => {
                        //Put directly to current node.
                        ret = node.entities.insert(entity);
                        break;
                    }
                }
            };
        }
        if ret {
            self.len += 1;
        }
        println!("counts {}", self.len());
        ret
    }

    ///Extend above root to cover given aabb.
    fn try_extend(&mut self, aabb: AABB) {
        if self.root == Self::NULL_INDEX {
            return;
        }
        self.base_aabb.extend_for(&aabb, |aabb| {
            let index = self.get_or_create_node(aabb, Self::NULL_INDEX);
            let prev_root_aabb = self.nodes[self.root].aabb;
            let mut node = &mut self.nodes[index];
            let octant = (prev_root_aabb - node.aabb.center())
                .octant()
                .expect("Maybe float point precision problem");
            let octant_index = OctreeNodeA::octant_to_index(octant);
            node.children[octant_index] = index;
            self.base_aabb = aabb;
            self.root = index;
        });
    }

    ///Return is whether existed entity is removed.
    pub fn remove(&mut self, entity: Entity, aabb: AABB) -> bool {
        let entity = OctreeEntity::new(entity, aabb);
        let mut index = self.root;
        let mut octant_index = Self::NULL_INDEX;
        let mut ret = false;
        loop {
            if index == Self::NULL_INDEX {
                //When tree traversal met dead end.
                break;
            }
            let node = &mut self.nodes[index];
            if node.children_len == 0 {
                //When node has no child.
                ret = node.entities.remove(&entity);
                if node.entities.is_empty() {
                    //Makes node idle when it is totally empty.
                    self.idles_node(index, octant_index);
                }
                break;
            } else {
                //Whether entity is fit in node's arbitrary octant.
                match (entity.aabb - node.aabb.center()).octant() {
                    Some(octant) => {
                        octant_index = OctreeNodeA::octant_to_index(octant);
                        index = node.children[octant_index];
                    }
                    None => {
                        ret = node.entities.remove(&entity);
                        break;
                    }
                }
            }
        }
        if ret {
            self.len -= 1;
        }
        println!("counts {}", self.len());
        ret
    }

    ///// Iterating entities that intersects with given bounding box.
    // pub fn intersect(&self, aabb: AABB, f: impl Fn(&Entity)) {
    //     let mut index = self.root;
    //     loop {
    //         if index == Self::NULL_INDEX {
    //             break;
    //         }
    //         let node = &mut self.nodes[index];
    //         for entity in node.entities.iter() {
    //             if entity.aabb.intersects(&aabb) {
    //                 f(&entity.entity);
    //             }
    //         }
    //         match (aabb - node.aabb.center()).octant() {
    //             Some(octant) => {
    //                 let octant_index = OctreeNode::octant_to_index(octant);
    //                 index = node.children[octant_index];
    //             }
    //             _ => {}
    //         }
    //     }
    // }
}

pub struct OctreeNodeA {
    ///Bound of itself.
    aabb: AABB,
    ///Entities that a few or doesn't fit with childs.
    entities: BTreeSet<OctreeEntity>,
    parent: usize,
    children: [usize; 8],
    children_len: usize,
}

impl OctreeNodeA {
    pub fn new(aabb: AABB, parent: usize) -> Self {
        Self {
            aabb,
            entities: BTreeSet::new(),
            parent,
            children: [Octree::NULL_INDEX; 8],
            children_len: 0,
        }
    }

    ///Quick conversion from octant to children leaf index.
    const fn octant_to_index(octant: BVec3) -> usize {
        const STEP_X: usize = 4;
        const STEP_Y: usize = 2;
        const STEP_Z: usize = 1;
        STEP_X * octant.x as usize + STEP_Y * octant.y as usize + STEP_Z * octant.z as usize
    }
}

///Octree that 3 dimensional version of binary heap. Useful for broad collision via aabb.
#[derive(Component)]
pub struct OctreeNode {
    ///Bound of itself.
    pub bound: AABB,
    ///Entities that a few or doesn't fit with leaves.
    entities: BTreeSet<OctreeEntity>,
    ///Leaf nodes that divides space.
    leaves: Option<Box<[OctreeNode; 8]>>,
    ///Total amounts of entities below.
    length: usize,
}

impl OctreeNode {
    const SPLIT_THRESHOLD: usize = 4;

    pub fn new(size: f32, offset: Vec3) -> OctreeNode {
        Self::from_bound(AABB::from_size_offset(size, offset))
    }

    pub fn from_bound(bound: AABB) -> Self {
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
    pub fn insert(&mut self, entity: Entity, bound: AABB) -> bool {
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
                    Some(leaves) => match (entity.aabb - self.bound.center()).octant() {
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
            |&entity| match (entity.aabb - self.bound.center()).octant() {
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
    pub fn remove(&mut self, entity: Entity, bound: AABB) -> bool {
        let ret = self.remove_inner(OctreeEntity::new(entity, bound));
        println!("counts {}", self.len());
        ret
    }

    fn remove_inner(&mut self, entity: OctreeEntity) -> bool {
        let ret = if let Some(ref mut leaves) = self.leaves {
            match (entity.aabb - self.bound.center()).octant() {
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
    pub fn intersect(&self, bound: AABB, f: impl Fn(&Entity)) {
        for entity in self.entities.iter() {
            if entity.aabb.intersects(&bound) {
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

    pub fn raycast_hit(&self, ray: Ray, correction: f32) -> Option<(Entity, AABB, Vec3)> {
        let mut len = f32::INFINITY;
        match self.raycast_inner(ray, &mut len, &mut 0.) {
            Some((e, b)) => Some((e, b, ray.origin + ray.dir * (len - correction))),
            None => None,
        }
    }

    pub fn raycast(&self, ray: Ray) -> Option<(Entity, AABB, f32)> {
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
    ) -> Option<(Entity, AABB)> {
        //At least, ray should intersect node's bound
        if let Some((_, t_max)) = self.bound.intersects_ray_raw(ray) {
            let mut ret = None;
            //Checking all containing entities.
            for entity in self.entities.iter() {
                if let Some(candidate) = entity.aabb.intersects_ray(ray) {
                    //result should be the shortest one.
                    if candidate < *len {
                        ret = Some((entity.entity, entity.aabb));
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
