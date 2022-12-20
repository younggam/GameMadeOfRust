use crate::physics::{
    aabb::AABB,
    collider::Collider,
    collider::Shape,
    ray::{Ray, RayHitInfo},
};

use std::{borrow::Borrow, cmp::Ordering, collections::BTreeSet};

use bevy::prelude::*;

///Caching data for octree to prevent frequent recalculate.
#[derive(Clone)]
pub struct OctreeEntity {
    entity: Entity,
    aabb: AABB,
    shape: Shape,
    rotation: Quat,
}

impl OctreeEntity {
    pub fn new(entity: Entity, collider: &Collider, transform: &Transform) -> Self {
        Self {
            entity,
            aabb: collider.aabb(transform),
            shape: collider.shape(),
            rotation: transform.rotation,
        }
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

impl Borrow<Entity> for OctreeEntity {
    fn borrow(&self) -> &Entity {
        &self.entity
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
    pub fn _is_empty(&self) -> bool {
        self.len == 0
    }

    ///Root node aabb.
    pub fn _base_aabb(&self) -> &AABB {
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
    pub fn insert(&mut self, entity: OctreeEntity) -> bool {
        self.try_extend(&entity.aabb);
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
            self.base_aabb = self.base_aabb.extend(aabb);
        } else {
            self.base_aabb.extend_for(aabb, |aabb| {
                println!("extend");
                let index = self.get_or_create_node(aabb, Self::NULL_INDEX);
                let octant = (self.nodes[self.root].aabb - self.nodes[index].aabb.center())
                    .octant()
                    .expect("Maybe float point precision problem");
                self.nodes[self.root].parent = index;
                let parent = &mut self.nodes[index];
                parent.children_len += 1;
                parent.children[OctreeNode::octant_to_index(octant)] = self.root;
                self.base_aabb = aabb;
                self.root = index;
            });
        }
    }

    ///Return is whether existed entity is removed.
    pub fn remove(&mut self, entity: Entity, aabb: AABB) -> bool {
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
                match (aabb - node.aabb.center()).octant() {
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
    pub fn _intersect(&self, aabb: AABB, f: impl Fn(&Entity)) {
        let mut index = self.root;
        while index != Self::NULL_INDEX {
            let node = &self.nodes[index];
            for entity in node.entities.iter() {
                if entity.aabb._intersects(&aabb) {
                    f(&entity.entity);
                }
            }
            match (aabb - node.aabb.center()).octant() {
                Some(octant) => {
                    //Go deep until entity does not fit with leaf.
                    index = node.get_child_index(octant);
                }
                None => {
                    self._intersect_children(&index, &aabb, &f);
                    break;
                }
            }
        }
    }

    ///When entity has possibility to intersect with all leaves below.
    fn _intersect_children(&self, index: &usize, aabb: &AABB, f: &impl Fn(&Entity)) {
        //Iterates all possible child.
        for child_index in self.nodes[*index].children.iter() {
            if *child_index == Self::NULL_INDEX {
                continue;
            }
            let child = &self.nodes[*child_index];
            if child.aabb._intersects(&aabb) {
                for entity in child.entities.iter() {
                    if entity.aabb._intersects(&aabb) {
                        f(&entity.entity);
                    }
                }
                self._intersect_children(child_index, aabb, f);
            }
        }
    }

    ///Return hit information about raycast.
    pub fn raycast(&self, ray: &Ray) -> Option<RayHitInfo> {
        let mut len = f32::INFINITY;
        let mut pivot = 0f32;
        self.raycast_inner(self.root, ray, &mut len, &mut pivot)
            .map(|(e, b)| RayHitInfo::new(e, b, len))
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
