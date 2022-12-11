use crate::{
    asset::*,
    consts::*,
    physics::{Ray, *},
    states::*,
    ui::*,
};

use bevy::{input::mouse::MouseMotion, prelude::*, window::CursorGrabMode};

use bevy_polyline::prelude::*;

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
            transform: Transform::from_scale(Vec3::new(100., 1., 100.)),
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
    commands.spawn((OctreeNode::new(64., Vec3::new(0., 32., 0.)), state.mark()));
    //selection
    let selection = Selection::new(
        meshs[MESH_BUILT_IN][CUBE].clone(),
        standard_materials[S_MAT_BUILT_IN][WHITE].clone(),
        standard_materials[S_MAT_BUILT_IN][WHITE_TRANS].clone(),
        AABB::from_size(1.),
    );
    commands.spawn((
        PbrBundle {
            mesh: selection.mesh.clone(),
            material: selection.material_trans.clone(),
            visibility: Visibility::INVISIBLE,
            ..default()
        },
        selection,
        state.mark(),
    ));
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
        transform.translation += to_move.clamp_length_max(1.0) * delta;
    }
}

#[derive(Component)]
pub struct LookAt(Option<(Option<(Entity, AABB)>, Vec3)>);

#[derive(Component)]
pub struct Selection {
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
    material_trans: Handle<StandardMaterial>,
    bound: AABB,
}

impl Selection {
    pub fn new(
        mesh: Handle<Mesh>,
        material: Handle<StandardMaterial>,
        material_trans: Handle<StandardMaterial>,
        bound: AABB,
    ) -> Self {
        Self {
            mesh,
            material,
            material_trans,
            bound,
        }
    }
}

fn select(
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
    octree: Query<&OctreeNode>,
    mut selection: Query<(&mut Transform, &mut Visibility), (With<Selection>, Without<Camera>)>,
) {
    let (transform, mut look_at) = camera.single_mut();
    let camera_pos = transform.translation;
    let camera_forward = transform.forward();
    let octree = octree.single();
    let mut selection = selection.single_mut();
    fn set_selection(pos: Vec3, mut selection: (Mut<Transform>, Mut<Visibility>)) -> Vec3 {
        let pos = pos.floor() + 0.5;
        selection.0.translation = pos;
        *selection.1 = Visibility::VISIBLE;
        pos
    }
    //Get raycast hit point.
    look_at.0 = match octree.raycast_hit(Ray::new(camera_pos, camera_forward), 0.001) {
        Some((e, b, p)) => Some((Some((e, b)), set_selection(p, selection))),
        //If no result, checks root of tree's bound.
        None => match octree
            .bound
            .intersects_ray(Ray::new(camera_pos, camera_forward))
        {
            Some(len) => Some((
                None,
                set_selection(camera_pos + camera_forward * (len - 0.001), selection),
            )),
            None => {
                *selection.1 = Visibility::INVISIBLE;
                None
            }
        },
    };
}

///Places cube where camera looking at. Temporary.
fn place(
    mut commands: Commands,
    mut octree: Query<&mut OctreeNode>,
    camera: Query<&LookAt, With<Camera>>,
    state: Res<GlobalState>,
    selection: Query<&Selection>,
    input: Res<Input<MouseButton>>,
) {
    //Checks only when left click.
    if input.just_pressed(MouseButton::Left) {
        if let Some((_, pos)) = camera.single().0 {
            let selection = selection.single();
            //If there's a result, spawn a cube.
            let entity = commands
                .spawn((
                    PbrBundle {
                        mesh: selection.mesh.clone(),
                        material: selection.material.clone(),
                        transform: Transform::from_translation(pos),
                        ..default()
                    },
                    state.mark(),
                    Collides,
                    selection.bound,
                ))
                .id();
            octree.single_mut().insert(entity, selection.bound + pos);
        }
    }
}

///Replaces cube where camera looking at. Temporary.
fn replace(
    mut commands: Commands,
    mut octree: Query<&mut OctreeNode>,
    camera: Query<&LookAt, With<Camera>>,
    input: Res<Input<MouseButton>>,
) {
    //Checks only when right click.
    if input.just_pressed(MouseButton::Right) {
        if let Some((Some((e, b)), _)) = camera.single().0 {
            //If there's a result, despawn a cube.
            octree.single_mut().remove(e, b);
            commands.entity(e).despawn_recursive();
        }
    }
}
