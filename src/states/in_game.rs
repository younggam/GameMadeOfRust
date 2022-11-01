use crate::{consts::*, states::*, ui::*};

use bevy::{input::mouse::MouseMotion, prelude::*};

pub struct InGamePlugin;

impl Plugin for InGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set_to_stage(
            CoreStage::PreUpdate,
            SystemSet::on_enter(PreUpdateStageState::InGame).with_system(setup),
        )
        .add_system_set_to_stage(
            CoreStage::Update,
            SystemSet::on_update(UpdateStageState::InGame)
                .with_system(move_camera)
                .with_system(close_requested),
        );
    }
}

fn move_camera(
    mut query: Query<(&Camera, &mut Transform)>,
    input: Res<Input<KeyCode>>,
    mut mouse: EventReader<MouseMotion>,
    time: Res<Time>,
) {
    let delta = time.delta_seconds();
    let mut motion = Vec2::ZERO;
    if !mouse.is_empty() {
        mouse.iter().for_each(|m| motion += m.delta);
        motion *= -delta;
    }

    let delta = delta * 10.0;
    for (_, mut transform) in query.iter_mut() {
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

fn setup(
    mut commands: Commands,
    state: Res<GlobalState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands
        .spawn_bundle(Camera3dBundle {
            transform: Transform::from_xyz(-4.0, 100.0, -5.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        })
        .insert(state.mark());
    // lightR
    commands
        .spawn_bundle(PointLightBundle {
            point_light: PointLight {
                intensity: 1500.0,
                shadows_enabled: true,
                ..default()
            },
            transform: Transform::from_xyz(4.0, 8.0, 4.0),
            ..default()
        })
        .insert(state.mark());
    // plane
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane { size: 100.0 })),
            material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
            ..default()
        })
        .insert(state.mark());
}
