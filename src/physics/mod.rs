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
    pub fn intersects(&self, other: &Self) -> bool {
        self.min.cmplt(other.max).all() && self.max.cmpgt(other.min).all()
    }

    ///Checks whether point is in bounding box.
    pub fn overlaps_point(&self, point: Vec3) -> bool {
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
        let check = self.origin + pivot * self.dir - bound.get_octant(octant).center();
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

///A variation of Octree.
/// - There is no guarantee that children nodes are 8.
/// - Entity go or create leaf node if and only if it fit with leaf.
/// - This guarantees entity is on only one leaf.
/// - A leaf could have entities itself while having children.
/// - This has node pool that Empty leaf could be recycled.
#[derive(Component)]
pub struct Octree {
    ///Index of root node from pool.
    root: usize,
    ///Base aabb for creating root node.
    base_aabb: AABB,
    ///Kinda node pool
    nodes: Vec<OctreeNode>,
    ///Min leaf size to prevent too deep nodes.
    min_leaf_extent: Vec3,
    ///Index of idle root node from pool.
    idle: usize,
    len: usize,
}

impl Octree {
    const NULL_INDEX: usize = usize::MAX;

    pub fn new(capacity: usize, min_leaf_extent: Vec3, aabb: AABB) -> Self {
        Self {
            root: Self::NULL_INDEX,
            base_aabb: aabb,
            nodes: Vec::with_capacity(capacity),
            min_leaf_extent,
            idle: Self::NULL_INDEX,
            len: 0,
        }
    }

    pub fn from_size_offset(
        capacity: usize,
        min_leaf_extent: Vec3,
        size: f32,
        offset: Vec3,
    ) -> Self {
        Self::new(
            capacity,
            min_leaf_extent,
            AABB::from_size_offset(size, offset),
        )
    }

    pub fn len(&self) -> usize {
        self.len
    }

    ///If node and its leaves entirely empty.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    ///Root node aabb.
    pub fn base_aabb(&self) -> &AABB {
        &self.base_aabb
    }

    ///Create a node or find and set a idle node.
    fn get_or_create_node(&mut self, aabb: AABB, parent: usize) -> usize {
        if self.idle == Self::NULL_INDEX {
            //Create a node if there is no idle node.
            self.nodes.push(OctreeNode::new(aabb, parent));
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

    ///Idles empty node.
    ///Note: It doesn't idle empty parent node too.
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
        self.try_extend(&aabb);
        let mut index = self.root;
        let mut parent_index = Self::NULL_INDEX;
        let mut octant_index = Self::NULL_INDEX;
        let mut node_aabb = self.base_aabb;
        let ret;
        loop {
            if index == Self::NULL_INDEX {
                //Prevent tree to have too deep node.
                if self.min_leaf_extent.cmpgt(node_aabb.length()).any() {
                    ret = self.nodes[parent_index].entities.insert(entity);
                    break;
                }
                //When there is no next node, add new node into tree.
                index = self.get_or_create_node(node_aabb, parent_index);
                if parent_index == Self::NULL_INDEX {
                    self.root = index;
                } else {
                    //If there was parent, add child to it.
                    println!("split");
                    let parent = &mut self.nodes[parent_index];
                    parent.children_len += 1;
                    parent.children[octant_index] = index;
                }
            }
            let node = &mut self.nodes[index];
            //Whether entity is fit in node's arbitrary octant.
            match (entity.aabb - node.aabb.center()).octant() {
                Some(octant) => {
                    //Determine octant of child.
                    parent_index = index;
                    octant_index = OctreeNode::octant_to_index(octant);
                    node_aabb = node.aabb.get_octant(octant);
                    index = node.children[octant_index];
                }
                None => {
                    //Put directly to current node.
                    ret = node.entities.insert(entity);
                    break;
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
    fn try_extend(&mut self, aabb: &AABB) {
        if self.root == Self::NULL_INDEX {
            return;
        }
        self.base_aabb.extend_for(aabb, |aabb| {
            let index = self.get_or_create_node(aabb, Self::NULL_INDEX);
            let octant = (self.nodes[self.root].aabb - self.nodes[index].aabb.center())
                .octant()
                .expect("Maybe float point precision problem");
            self.nodes[self.root].parent = index;
            self.nodes[index].children[OctreeNode::octant_to_index(octant)] = self.root;
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
        //Stops when tree traversal met dead end.
        while index != Self::NULL_INDEX {
            let node = &mut self.nodes[index];
            if node.children_len == 0 {
                //When node has no child.
                ret = node.entities.remove(&entity);
                if node.entities.is_empty() {
                    //Makes node idle when it is totally empty.
                    self.idles_node(index, octant_index);
                    println!("unsplit");
                }
                break;
            } else {
                //Whether entity is fit in node's arbitrary octant.
                match (entity.aabb - node.aabb.center()).octant() {
                    Some(octant) => {
                        octant_index = OctreeNode::octant_to_index(octant);
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

    ///Iterating entities that intersects with given bounding box.
    pub fn intersect(&self, aabb: AABB, f: impl Fn(&Entity)) {
        let mut index = self.root;
        while index != Self::NULL_INDEX {
            let node = &self.nodes[index];
            for entity in node.entities.iter() {
                if entity.aabb.intersects(&aabb) {
                    f(&entity.entity);
                }
            }
            match (aabb - node.aabb.center()).octant() {
                Some(octant) => {
                    //Go deep until entity does not fit with leaf.
                    index = node.get_child_index(octant);
                }
                None => {
                    self.intersect_children(&index, &aabb, &f);
                    break;
                }
            }
        }
    }

    ///When entity has possibility to intersect with all leaves below.
    fn intersect_children(&self, index: &usize, aabb: &AABB, f: &impl Fn(&Entity)) {
        //Iterates all possible child.
        for child_index in self.nodes[*index].children.iter() {
            if *child_index == Self::NULL_INDEX {
                continue;
            }
            let child = &self.nodes[*child_index];
            if child.aabb.intersects(&aabb) {
                for entity in child.entities.iter() {
                    if entity.aabb.intersects(&aabb) {
                        f(&entity.entity);
                    }
                }
                self.intersect_children(child_index, aabb, f);
            }
        }
    }

    ///Return the bound of raycast have hit first and the hit point.
    pub fn raycast_hit(&self, ray: Ray, correction: f32) -> Option<(Entity, AABB, Vec3)> {
        self.raycast(ray)
            .map(|(e, b, len)| (e, b, ray.origin + ray.dir * (len - correction)))
    }

    ///Return the bound of raycast have hit first and its distance.
    pub fn raycast(&self, ray: Ray) -> Option<(Entity, AABB, f32)> {
        let mut len = f32::INFINITY;
        let mut pivot = 0f32;
        self.raycast_inner(self.root, &ray, &mut len, &mut pivot)
            .map(|(e, b)| (e, b, len))
    }

    fn raycast_inner(
        &self,
        index: usize,
        ray: &Ray,
        len: &mut f32,
        pivot: &mut f32,
    ) -> Option<(Entity, AABB)> {
        if index == Self::NULL_INDEX {
            None
        } else {
            let node = &self.nodes[index];
            //Ray should intersect at least node's aabb.
            match node.aabb.intersects_ray_raw(ray) {
                Some((_, t_max)) => {
                    let mut ret = None;
                    //Raycast entities in node itself.
                    for entity in node.entities.iter() {
                        if let Some(candidate) = entity.aabb.intersects_ray(ray) {
                            if candidate < *len {
                                ret = Some((entity.entity, entity.aabb));
                                *len = candidate;
                            }
                        }
                    }
                    //If node has child.
                    if node.children_len != 0 {
                        match ray.octant_at(*pivot, node.aabb) {
                            Some(mut octant) => loop {
                                let child_index = node.get_child_index(octant);
                                if child_index == Self::NULL_INDEX {
                                    //If child node doesn't exists, update just pivot.
                                    *pivot = match node
                                        .aabb
                                        .get_octant(octant)
                                        .intersects_ray_raw(ray)
                                    {
                                        Some((_, t_max)) => t_max,
                                        None => t_max,
                                    };
                                } else {
                                    //Get result of raycast on leaf.
                                    match self.raycast_inner(child_index, ray, len, pivot) {
                                        //First success is if and only if the shortest raycast on the leaves.
                                        tmp @ Some(_) => {
                                            ret = tmp;
                                            break;
                                        }
                                        None => {}
                                    }
                                }
                                //Shift leaf if there is still no result..
                                let prev_octant = octant;
                                octant = ray.next_octant(octant, *pivot, node.aabb);
                                //Dead end of ray through leaves.
                                if octant == prev_octant {
                                    break;
                                }
                            },
                            //Discard if ray is lie on xy, yz, xz plane and x, y, z axis
                            None => {}
                        }
                    }
                    //Update pivot to Next.
                    *pivot = t_max;
                    ret
                }
                None => None,
            }
        }
    }
}

pub struct OctreeNode {
    ///Bound of itself.
    aabb: AABB,
    ///Entities that a few or doesn't fit with childs.
    entities: BTreeSet<OctreeEntity>,
    parent: usize,
    children: [usize; 8],
    children_len: usize,
}

impl OctreeNode {
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

    pub fn get_child_index(&self, octant: BVec3) -> usize {
        self.children[Self::octant_to_index(octant)]
    }
}
