use crate::{func::*, states::*, ui::*};

use bevy::{app::AppExit, prelude::*};

pub struct InGamePlugin;

impl Plugin for InGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set_to_stage(
            CoreStage::PreUpdate,
            SystemSet::on_enter(PreUpdateStageState::InGame(None)).with_system(setup),
        )
        .add_system_set_to_stage(
            CoreStage::Update,
            SystemSet::on_update(UpdateStageState::InGame(None))
                .with_system(close_requested::<0, InGameState>),
        )
        .add_system_set_to_stage(
            CoreStage::PreUpdate,
            SystemSet::on_enter(PreUpdateStageState::InGame(Some(InGameState::AppExit)))
                .with_system(setup_exit),
        )
        .add_system_set_to_stage(
            CoreStage::Update,
            SystemSet::on_update(UpdateStageState::InGame(Some(InGameState::AppExit)))
                .with_system(exit_no_button::<0>)
                .with_system(exit_yes_button::<0>)
                .with_system(exit_close_requested),
        );
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
            transform: Transform::from_xyz(-4.0, 100.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
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

fn setup_exit(mut commands: Commands, state: Res<GlobalState>, asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(40.0), Val::Percent(24.0)),
                position_type: PositionType::Absolute,
                position: UiRect::new(
                    Val::Percent(30.0),
                    Val::Percent(70.0),
                    Val::Percent(62.0),
                    Val::Percent(38.0),
                ),
                flex_wrap: FlexWrap::WrapReverse,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                align_content: AlignContent::SpaceAround,
                ..default()
            },
            ..default()
        })
        .insert(state.mark())
        .with_children(|parent| {
            parent
                .spawn_bundle(NodeBundle {
                    style: Style {
                        flex_basis: Val::Percent(100.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn_bundle(create_text(
                        ARE_YOU_SURE_TEXT,
                        &asset_server,
                        30.0,
                        TEXT_COLOR_DARK,
                    ));
                });

            parent
                .spawn_bundle(create_button())
                .insert(Action::<for<'a> fn(&'a mut EventWriter<AppExit>)>::new(
                    |e: &mut EventWriter<AppExit>| e.send(AppExit),
                ))
                .insert(HierarchyMark::<0>)
                .with_children(|parent| {
                    parent.spawn_bundle(create_text(
                        YES_TEXT,
                        &asset_server,
                        30.0,
                        TEXT_COLOR_BRIGHT,
                    ));
                });

            parent
                .spawn_bundle(create_button())
                .insert(Action::<for<'a> fn(&'a mut GlobalState)>::new(
                    |g: &mut GlobalState| g.pop(),
                ))
                .insert(HierarchyMark::<0>)
                .with_children(|parent| {
                    parent.spawn_bundle(create_text(
                        NO_TEXT,
                        &asset_server,
                        30.0,
                        TEXT_COLOR_BRIGHT,
                    ));
                });
        });
}
