use crate::{
    asset::*,
    consts::*,
    physics::{aabb::AABB, octree::Octree, ray::Ray},
    states::*,
    ui::*,
};

use bevy::input::mouse::MouseWheel;
use bevy::{input::mouse::MouseMotion, prelude::*, window::CursorGrabMode};

use crate::physics::collider::{Collider, Shape};
use crate::physics::octree::OctreeEntity;
use crate::physics::ray::RayHitInfo;
use bevy_polyline::prelude::*;

const BLUEPRINT_BOUND: AABB =
    unsafe { AABB::new_unchecked(Vec3::new(-31.5, -0.5, -31.5), Vec3::new(31.5, 62.5, 31.5)) };

///Batch setup for In game.
pub struct InGamePlugin;

impl Plugin for InGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set_to_stage(
            CoreStage::PreUpdate,
            SystemSet::on_enter(PreUpdateStageState::InGame).with_system(setup),
        )
        .add_system_set_to_stage(
            CoreStage::PreUpdate,
            SystemSet::on_update(PreUpdateStageState::InGame)
                .with_system(grab_cursor)
                .with_system(camera_look_at),
        )
        .add_system_set_to_stage(
            CoreStage::PreUpdate,
            SystemSet::on_pause(PreUpdateStageState::InGame).with_system(show_cursor),
        )
        .add_system_set_to_stage(
            CoreStage::Update,
            SystemSet::on_update(UpdateStageState::InGame)
                .with_system(move_camera)
                .with_system(place)
                .with_system(replace)
                .with_system(close_requested),
        );
    }
}

///Setup system in game.
fn setup(
    mut commands: Commands,
    state: Res<GlobalState>,
    textures: Res<Images>,
    meshs: Res<Meshes>,
    standard_materials: Res<StandardMaterials>,
    polylines: Res<Polylines>,
    polyline_materials: Res<PolylineMaterials>,
    windows: Res<Windows>,
) {
    //camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(-4.0, 10.0, -5.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        state.mark(),
        LookAt(None),
    ));
    //crosshair
    let window = windows.primary();
    commands.spawn((
        ImageBundle {
            image: textures[IMAGE_UI][CROSSHAIR].clone().into(),
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
        },
        state.mark(),
    ));
    //directional light
    commands.spawn((
        DirectionalLightBundle {
            directional_light: DirectionalLight {
                illuminance: 32000.0,
                ..default()
            },
            transform: Transform {
                rotation: Quat::from_euler(EulerRot::ZYX, 0., PI * 0.25, -PI * 0.4),
                ..default()
            },
            ..default()
        },
        state.mark(),
    ));
    //plane
    commands.spawn((
        PbrBundle {
            mesh: meshs[MESH_BUILT_IN][PLANE].clone(),
            material: standard_materials[S_MAT_BUILT_IN][SEA_GREEN].clone(),
            transform: Transform::from_scale(Vec3::new(100., 1., 100.))
                .with_translation(Vec3::new(0., -0.5, 0.)),
            ..default()
        },
        state.mark(),
    ));
    //x axis line
    commands.spawn((
        PolylineBundle {
            polyline: polylines[UNIT_X].clone(),
            material: polyline_materials[RED].clone(),
            transform: Transform::from_scale(Vec3::new(100., 1., 1.)),
            ..default()
        },
        state.mark(),
    ));
    //y axis line
    commands.spawn((
        PolylineBundle {
            polyline: polylines[UNIT_X].clone(),
            material: polyline_materials[GREEN].clone(),
            transform: Transform::from_rotation(Quat::from_rotation_z(FRAC_PI_2))
                .with_scale(Vec3::new(100., 1., 1.)),
            ..default()
        },
        state.mark(),
    ));
    // z axis line
    commands.spawn((
        PolylineBundle {
            polyline: polylines[UNIT_X].clone(),
            material: polyline_materials[BLUE].clone(),
            transform: Transform::from_rotation(Quat::from_rotation_y(-FRAC_PI_2))
                .with_scale(Vec3::new(100., 1., 1.)),
            ..default()
        },
        state.mark(),
    ));
    //Octree
    commands.spawn((
        Octree::from_size_offset(64, Vec3::splat(0.9), 64., Vec3::new(0.5, 31.5, 0.5)),
        state.mark(),
    ));
    //selection
    let selection = Selection::new(
        vec![
            meshs[MESH_WEAPON][GUN_TOWER_0_BASE].clone(),
            meshs[MESH_WEAPON][GUN_TOWER_0_TOWER].clone(),
            meshs[MESH_WEAPON][GUN_TOWER_0_GUN].clone(),
        ],
        standard_materials[S_MAT_BUILT_IN][WHITE].clone(),
        standard_materials[S_MAT_BUILT_IN][WHITE_TRANS].clone(),
        Collider::from_shape(Shape::CutSphere {
            radius: 2.5,
            cut: 0.5,
        }),
    );
    let children = selection.create_transparent();
    commands
        .spawn((
            TransformBundle::default(),
            VisibilityBundle::default(),
            selection,
            state.mark(),
        ))
        .add_children(|parent| {
            for bundle in children {
                parent.spawn(bundle);
            }
        });
}

///locks cursor to window while in game.
fn grab_cursor(mut windows: ResMut<Windows>) {
    let window = windows.primary_mut();
    let cursor_visible = window.cursor_visible();
    if window.is_focused() {
        //if window is focused and cursor is visible, lock.
        if cursor_visible {
            window.set_cursor_grab_mode(CursorGrabMode::Locked);
            window.set_cursor_visibility(false);
        }
    }
    //if window isn't focused and cursor is invisible, release.
    else if !cursor_visible {
        window.set_cursor_grab_mode(CursorGrabMode::None);
        window.set_cursor_visibility(true);
    }
}

///Release cursor when about to exit.
fn show_cursor(mut windows: ResMut<Windows>) {
    let window = windows.primary_mut();
    window.set_cursor_grab_mode(CursorGrabMode::None);
    window.set_cursor_visibility(true);
}

///Camera control system.
fn move_camera(
    mut query: Query<&mut Transform, With<Camera>>,
    input: Res<Input<KeyCode>>,
    mut mouse: EventReader<MouseMotion>,
    time: Res<Time>,
) {
    //mouse motion to angular delta.
    let mut motion = Vec2::ZERO;
    if !mouse.is_empty() {
        mouse.iter().for_each(|m| motion += m.delta);
        motion *= -RADIANS * 0.08;
    }

    let delta = time.delta_seconds() * 10.0;
    for mut transform in query.iter_mut() {
        //camera rotation by mouse motion.
        if motion != Vec2::ZERO {
            let euler = transform.rotation.to_euler(EulerRot::YXZ);
            transform.rotation = Quat::from_euler(
                EulerRot::YXZ,
                motion.x + euler.0,
                (motion.y + euler.1).clamp(-GIMBAL_LOCK, GIMBAL_LOCK),
                0.0,
            );
        }
        //Accumulate move direction from keyboard inputs.
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
        //apply
        transform.translation = (transform.translation + to_move.clamp_length_max(1.0) * delta)
            .clamp(BLUEPRINT_BOUND.min() + 0.5, BLUEPRINT_BOUND.max() - 0.5);
    }
}

#[derive(Component)]
pub struct LookAt(Option<RayHitInfo>);

#[derive(Component)]
pub struct Selection {
    valid: bool,
    meshes: Vec<Handle<Mesh>>,
    material: Handle<StandardMaterial>,
    material_trans: Handle<StandardMaterial>,
    collider: Collider,
}

impl Selection {
    pub fn new(
        meshes: Vec<Handle<Mesh>>,
        material: Handle<StandardMaterial>,
        material_trans: Handle<StandardMaterial>,
        collider: Collider,
    ) -> Self {
        Self {
            valid: false,
            meshes,
            material,
            material_trans,
            collider,
        }
    }

    pub fn create_transparent(&self) -> Vec<PbrBundle> {
        self.meshes
            .iter()
            .map(|mesh| PbrBundle {
                mesh: mesh.clone(),
                material: self.material_trans.clone(),
                ..default()
            })
            .collect()
    }

    pub fn create(&self) -> Vec<PbrBundle> {
        self.meshes
            .iter()
            .map(|mesh| PbrBundle {
                mesh: mesh.clone(),
                material: self.material.clone(),
                ..default()
            })
            .collect()
    }
}

fn _select(
    mut selected: Query<(
        &mut Handle<Mesh>,
        &mut Handle<StandardMaterial>,
        &mut Selection,
    )>,
) {
    let _ = selected.single_mut();
}

///Prepare and store data about where camera looking at.
fn camera_look_at(
    mut camera: Query<(&Transform, &mut LookAt), With<Camera>>,
    octree: Query<&Octree>,
    mut selection: Query<(&mut Selection, &mut Transform), Without<Camera>>,
    mut mouse_wheel: EventReader<MouseWheel>,
    mut rotate: Local<i32>,
) {
    let mut accum = 0.;
    for delta in mouse_wheel.iter() {
        accum += delta.y;
    }
    if accum > 0. {
        *rotate += 1
    } else if accum < 0. {
        *rotate -= 1
    }
    let y_rot = (*rotate % 4) as f32 * 90f32.to_radians();

    let (camera_transform, mut look_at) = camera.single_mut();
    let camera_pos = camera_transform.translation;
    let camera_forward = camera_transform.forward();
    let octree = octree.single();
    let (mut selection, mut transform) = selection.single_mut();
    //Get raycast hit point.
    let ray = Ray::new(camera_pos, camera_forward);
    look_at.0 = match octree.raycast(&ray) {
        Some(hit_info) => {
            let pos = ray.point(hit_info.t + 0.001);
            let face = hit_info.aabb.face(pos);
            transform.translation = pos.round() + face;
            transform.rotation =
                Quat::from_rotation_arc(Vec3::Y, face) * Quat::from_rotation_y(y_rot);
            selection.valid = true;
            Some(hit_info)
        }
        //If no result, checks root of tree's bound.
        None => match BLUEPRINT_BOUND.intersects_ray(&ray) {
            Some(len) => {
                let pos = ray.point(len + 0.001);
                let face = -BLUEPRINT_BOUND.face(pos);
                transform.translation = pos.round() + face;
                transform.rotation =
                    Quat::from_rotation_arc(Vec3::Y, face) * Quat::from_rotation_y(y_rot);
                selection.valid = true;
                None
            }
            None => {
                selection.valid = false;
                None
            }
        },
    };
}

///Places cube where camera looking at. Temporary.
fn place(
    mut commands: Commands,
    mut octree: Query<&mut Octree>,
    state: Res<GlobalState>,
    selection: Query<(&Selection, &Transform)>,
    input: Res<Input<MouseButton>>,
    time: Res<Time>,
    mut press_time: Local<f32>,
) {
    //Checks only when left click.
    let mut place = input.just_pressed(MouseButton::Left);
    if !place {
        //Repeat place if button is pressed long enough.
        if input.pressed(MouseButton::Left) {
            *press_time += time.delta_seconds();
            if *press_time >= 1. {
                place = true;
                *press_time -= 0.1;
            }
        } else {
            *press_time = 0.;
        }
    }

    let (selection, &transform) = selection.single();
    if place {
        if selection.valid {
            //If there's a result, spawn a selection.
            let children = selection.create();
            let entity = commands
                .spawn((
                    TransformBundle {
                        local: transform,
                        ..default()
                    },
                    VisibilityBundle::default(),
                    state.mark(),
                    selection.collider.clone(),
                ))
                .with_children(|parent| {
                    for bundle in children {
                        parent.spawn(bundle);
                    }
                })
                .id();
            octree
                .single_mut()
                .insert(OctreeEntity::new(entity, &selection.collider, &transform));
        }
    }
}

///Replaces cube where camera looking at. Temporary.
fn replace(
    mut commands: Commands,
    mut octree: Query<&mut Octree>,
    camera: Query<&LookAt, With<Camera>>,
    input: Res<Input<MouseButton>>,
    time: Res<Time>,
    mut press_time: Local<f32>,
) {
    //Checks only when right click.
    let mut replace = input.just_pressed(MouseButton::Right);
    if !replace {
        //Repeat place if button is pressed long enough.
        if input.pressed(MouseButton::Right) {
            *press_time += time.delta_seconds();
            if *press_time >= 1. {
                replace = true;
                *press_time -= 0.1;
            }
        } else {
            *press_time = 0.;
        }
    }

    if replace {
        if let Some(hit_info) = &camera.single().0 {
            //If there's a result, despawn a cube.
            if octree.single_mut().remove(hit_info.entity, hit_info.aabb){
                commands.entity(hit_info.entity).despawn_recursive();
            }
        }
    }
}
