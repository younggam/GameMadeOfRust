use crate::{consts::*, states::*, ui::*, Textures};
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::ops::{Add, Deref, Sub};

use bevy::math::DVec3;
use bevy::{
    input::mouse::MouseMotion,
    prelude::{shape::Plane, *},
};

use bevy_polyline::prelude::*;

pub struct InGamePlugin;

impl Plugin for InGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set_to_stage(
            CoreStage::PreUpdate,
            SystemSet::on_enter(PreUpdateStageState::InGame).with_system(setup),
        )
        .add_system_set_to_stage(
            CoreStage::PreUpdate,
            SystemSet::on_update(PreUpdateStageState::InGame).with_system(grab_cursor),
        )
        .add_system_set_to_stage(
            CoreStage::PreUpdate,
            SystemSet::on_pause(PreUpdateStageState::InGame).with_system(show_cursor),
        )
        .add_system_set_to_stage(
            CoreStage::Update,
            SystemSet::on_update(UpdateStageState::InGame)
                .with_system(move_camera)
                .with_system(close_requested),
        );
    }
}

fn setup(
    mut commands: Commands,
    state: Res<GlobalState>,
    textures: Res<Textures>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut polylines: ResMut<Assets<Polyline>>,
    mut polyline_materials: ResMut<Assets<PolylineMaterial>>,
    windows: Res<Windows>,
) {
    // camera
    commands
        .spawn_bundle(Camera3dBundle {
            transform: Transform::from_xyz(-4.0, 10.0, -5.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        })
        .insert(state.mark());
    // crosshair
    let window = windows.primary();
    commands
        .spawn_bundle(ImageBundle {
            image: textures[UI][UI_CROSSHAIR].clone().into(),
            style: Style {
                size: Size::new(Val::Px(32.), Val::Px(32.)),
                position_type: PositionType::Absolute,
                position: UiRect::new(
                    Val::Px(window.width() * 0.5 - 16.),
                    Val::Undefined,
                    Val::Undefined,
                    Val::Px(window.height() * 0.5 - 16.),
                ),
                ..default()
            },
            ..default()
        })
        .insert(state.mark());
    // directional light
    commands
        .spawn_bundle(DirectionalLightBundle {
            directional_light: DirectionalLight {
                illuminance: 32000.0,
                ..default()
            },
            transform: Transform {
                rotation: Quat::from_rotation_x(-std::f32::consts::PI * 0.5),
                ..default()
            },
            ..default()
        })
        .insert(state.mark());
    // plane
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Plane { size: 100.0 }.into()),
            material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
            ..default()
        })
        .insert(state.mark());
    // x axis line
    commands
        .spawn_bundle(PolylineBundle {
            polyline: polylines.add(Polyline {
                vertices: vec![Vec3::ZERO, Vec3::X * 100.],
                ..default()
            }),
            material: polyline_materials.add(PolylineMaterial {
                color: Color::RED,
                perspective: true,
                ..default()
            }),
            ..default()
        })
        .insert(state.mark());
    // y axis line
    commands
        .spawn_bundle(PolylineBundle {
            polyline: polylines.add(Polyline {
                vertices: vec![Vec3::ZERO, Vec3::Y * 100.],
                ..default()
            }),
            material: polyline_materials.add(PolylineMaterial {
                color: Color::GREEN,
                perspective: true,
                ..default()
            }),
            ..default()
        })
        .insert(state.mark());
    // z axis line
    commands
        .spawn_bundle(PolylineBundle {
            polyline: polylines.add(Polyline {
                vertices: vec![Vec3::ZERO, Vec3::Z * 100.],
                ..default()
            }),
            material: polyline_materials.add(PolylineMaterial {
                color: Color::BLUE,
                perspective: true,
                ..default()
            }),
            ..default()
        })
        .insert(state.mark());
    // Octree
    commands
        .spawn()
        .insert(OctreeNode::new(64., Vec3::new(0., 32., 0.)))
        .insert(state.mark());
}

fn grab_cursor(mut windows: ResMut<Windows>) {
    let window = windows.primary_mut();
    let cursor_visible = window.cursor_visible();
    if window.is_focused() {
        if cursor_visible {
            window.set_cursor_lock_mode(true);
            window.set_cursor_visibility(false);
        }
    } else if !cursor_visible {
        window.set_cursor_lock_mode(false);
        window.set_cursor_visibility(true);
    }
}

fn show_cursor(mut windows: ResMut<Windows>) {
    let window = windows.primary_mut();
    window.set_cursor_lock_mode(false);
    window.set_cursor_visibility(true);
}

fn move_camera(
    mut query: Query<&mut Transform, With<Camera>>,
    input: Res<Input<KeyCode>>,
    mut mouse: EventReader<MouseMotion>,
    cursor: EventReader<CursorMoved>,
    time: Res<Time>,
) {
    let delta = time.delta_seconds();
    let mut motion = Vec2::ZERO;
    if !mouse.is_empty() && !cursor.is_empty() {
        mouse.iter().for_each(|m| motion += m.delta);
        motion *= -RADIANS * 0.08;
    }

    let delta = delta * 10.0;
    for mut transform in query.iter_mut() {
        if motion != Vec2::ZERO {
            let euler = transform.rotation.to_euler(EulerRot::YXZ);
            transform.rotation = Quat::from_euler(
                EulerRot::YXZ,
                motion.x + euler.0,
                (motion.y + euler.1).clamp(-GIMBAL_LOCK, GIMBAL_LOCK),
                0.0,
            );
        }
        let front = transform.forward();
        let right = transform.right();
        let up = Vec3::Y;
        let mut to_move = Vec3::ZERO;
        if input.any_pressed([KeyCode::W, KeyCode::Up]) {
            to_move += front;
        }
        if input.any_pressed([KeyCode::A, KeyCode::Left]) {
            to_move -= right;
        }
        if input.any_pressed([KeyCode::S, KeyCode::Down]) {
            to_move -= front;
        }
        if input.any_pressed([KeyCode::D, KeyCode::Right]) {
            to_move += right;
        }
        if input.pressed(KeyCode::Space) {
            to_move += up;
        }
        if input.pressed(KeyCode::LShift) {
            to_move -= up;
        }
        transform.translation += to_move.clamp_length_max(1.0) * delta;
    }
}

#[derive(Component)]
pub struct Collides;

#[derive(Component, Clone, Copy)]
pub struct BoundingBox {
    min: Vec3,
    max: Vec3,
}

impl BoundingBox {
    pub fn new(min: Vec3, max: Vec3) -> Self {
        if min.cmpgt(max).any() {
            panic!("min value of BoundingBox is bigger than max");
        }
        Self { min, max }
    }

    pub fn from_size(size: f32, offset: Vec3) -> Self {
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

    pub fn octants(&self) -> Option<BVec3> {
        let x_p = self.min.x > 0. && self.max.x > 0.;
        let x_n = self.min.x < 0. && self.max.x < 0.;
        let y_p = self.min.y > 0. && self.max.y > 0.;
        let y_n = self.min.y < 0. && self.max.y < 0.;
        let z_p = self.min.z > 0. && self.max.z > 0.;
        let z_n = self.min.z < 0. && self.max.z < 0.;
        if x_p ^ x_n && y_p ^ y_n && z_p ^ z_n {
            Some(BVec3::new(x_p, y_p, z_p))
        } else {
            None
        }
    }

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

#[derive(Copy, Clone)]
struct OctreeEntity {
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

#[derive(Component)]
pub struct OctreeNode {
    bound: BoundingBox,
    entities: BTreeSet<OctreeEntity>,
    leaves: BTreeMap<u32, OctreeNode>,
}

impl OctreeNode {
    pub fn new(size: f32, offset: Vec3) -> OctreeNode {
        Self::from_bound(BoundingBox::from_size(size, offset))
    }

    pub fn from_bound(bound: BoundingBox) -> Self {
        OctreeNode {
            bound,
            entities: BTreeSet::new(),
            leaves: BTreeMap::new(),
        }
    }

    pub fn removal_system(
        mut octree: Query<&mut OctreeNode>,
        removals: RemovedComponents<Collides>,
        query: Query<(&Transform, &BoundingBox), Added<Collides>>,
    ) {
        let mut octree = octree.single_mut();
        for entity in removals.iter() {
            let (transform, bound) = query.get(entity).expect("Ghost entity");
            octree.remove(entity, *bound + transform.translation);
        }
    }

    pub fn insertion_system(
        mut octree: Query<&mut OctreeNode>,
        query: Query<(Entity, &Transform, &BoundingBox), Added<Collides>>,
    ) {
        let mut octree = octree.single_mut();
        for (entity, transform, bound) in query.iter() {
            octree.insert(entity, *bound + transform.translation);
        }
    }

    pub fn collision_system() {}

    const fn octant_to_index(&self, x: bool, y: bool, z: bool) -> u32 {
        const STEP_X: u32 = 4;
        const STEP_Y: u32 = 2;
        const STEP_Z: u32 = 1;
        STEP_X * x as u32 + STEP_Y * y as u32 + STEP_Z * z as u32
    }

    pub fn insert(&mut self, entity: Entity, bound: BoundingBox) -> bool {
        self.insert_inner(OctreeEntity::new(entity, bound))
    }

    fn insert_inner(&mut self, entity: OctreeEntity) -> bool {
        // kinda threshold
        if self.entities.len() >= 4 {
            let center = self.bound.center();
            let mut to_leaves =
                Vec::<(OctreeEntity, bool, bool, bool)>::with_capacity(self.entities.len() + 1);

            let mut ret = match (entity.bound - center).octants() {
                Some(BVec3 { x, y, z }) => {
                    to_leaves.push((entity, x, y, z));
                    true
                }
                None => self.entities.insert(entity),
            };

            self.entities
                .retain(|&entity| match (entity.bound - center).octants() {
                    Some(BVec3 { x, y, z }) => {
                        to_leaves.push((entity, x, y, z));
                        false
                    }
                    None => true,
                });

            for (entity, x, y, z) in to_leaves {
                let i = self.octant_to_index(x, y, z);
                ret &= match self.leaves.get_mut(&i) {
                    Some(leaf) => leaf.insert_inner(entity),
                    None => {
                        let mut new_leaf = Self::from_bound(self.bound.get_octant(x, y, z));
                        new_leaf.insert_inner(entity);
                        self.leaves.insert(i, new_leaf);
                        true
                    }
                };
            }
            ret
        } else {
            self.entities.insert(entity)
        }
    }

    pub fn remove(&mut self, entity: Entity, bound: BoundingBox) -> bool {
        self.remove_inner(OctreeEntity::new(entity, bound))
    }

    fn remove_inner(&mut self, entity: OctreeEntity) -> bool {
        match (entity.bound - self.bound.center()).octants() {
            Some(BVec3 { x, y, z }) => {
                let i = self.octant_to_index(x, y, z);
                match self.leaves.get_mut(&i) {
                    Some(leaf) => {
                        let ret = leaf.remove_inner(entity);
                        if ret && leaf.entities.is_empty() {
                            self.leaves.remove(&i);
                        }
                        ret
                    }
                    None => self.entities.remove(&entity),
                }
            }
            None => self.entities.remove(&entity),
        }
    }
}
