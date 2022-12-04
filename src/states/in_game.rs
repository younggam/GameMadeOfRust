use crate::{
    asset::{Textures, UI, UI_CROSSHAIR},
    consts::*,
    physics::{Ray, *},
    states::*,
    ui::*,
};

use std::f32::consts::PI;

use bevy::window::CursorGrabMode;
use bevy::{
    input::mouse::MouseMotion,
    prelude::{
        shape::{Cube, Plane},
        *,
    },
};

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
    textures: Res<Textures>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut polylines: ResMut<Assets<Polyline>>,
    mut polyline_materials: ResMut<Assets<PolylineMaterial>>,
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
            mesh: meshes.add(Plane { size: 100.0 }.into()),
            material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
            ..default()
        },
        state.mark(),
    ));
    //x axis line
    commands.spawn((
        PolylineBundle {
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
        },
        state.mark(),
    ));
    //y axis line
    commands.spawn((
        PolylineBundle {
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
        },
        state.mark(),
    ));
    // z axis line
    commands.spawn((
        PolylineBundle {
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
        },
        state.mark(),
    ));
    //Octree
    commands.spawn((OctreeNode::new(64., Vec3::new(0., 32., 0.)), state.mark()));
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
pub struct LookAt(Option<(Option<(Entity, BoundingBox)>, Vec3)>);

///Prepare and store data about where camera looking at.
fn camera_look_at(
    mut camera: Query<(&Transform, &mut LookAt), With<Camera>>,
    octree: Query<&OctreeNode>,
) {
    let (transform, mut look_at) = camera.single_mut();
    let camera_pos = transform.translation;
    let camera_forward = transform.forward();
    let octree = octree.single();
    //Get raycast hit point.
    look_at.0 = match octree.raycast_hit(Ray::new(camera_pos, camera_forward), 0.01) {
        Some((e, b, p)) => Some((Some((e, b)), p.floor())),
        //If no result, checks root of tree's bound.
        None => match octree
            .bound
            .intersects_ray(Ray::new(camera_pos, camera_forward))
        {
            Some(len) => Some((None, (camera_pos + camera_forward * (len - 0.01)).floor())),
            None => None,
        },
    };
}

///Places cube where camera looking at. Temporary.
fn place(
    mut commands: Commands,
    mut octree: Query<&mut OctreeNode>,
    camera: Query<&LookAt, With<Camera>>,
    state: Res<GlobalState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    input: Res<Input<MouseButton>>,
) {
    //Checks only when left click.
    if input.just_pressed(MouseButton::Left) {
        if let Some((_, mut p)) = camera.single().0 {
            p += Vec3::splat(0.5);
            let b = BoundingBox::from_size(1.);
            //If there's a result, spawn a cube.
            let entity = commands
                .spawn((
                    PbrBundle {
                        mesh: meshes.add(Cube::new(1.).into()),
                        material: materials.add(Color::WHITE.into()),
                        transform: Transform::from_translation(p),
                        ..default()
                    },
                    state.mark(),
                    Collides,
                    b,
                ))
                .id();
            octree.single_mut().insert(entity, b + p);
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
