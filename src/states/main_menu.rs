use crate::{func::*, states::*, ui::*};

use bevy::{app::AppExit, prelude::*};

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        //MainMenu
        app.add_system_set_to_stage(
            CoreStage::PreUpdate,
            SystemSet::on_enter(PreUpdateStageState::MainMenu(None)).with_system(setup),
        )
        .add_system_set_to_stage(
            CoreStage::Update,
            SystemSet::on_update(UpdateStageState::MainMenu(None))
                .with_system(button)
                .with_system(close_requested::<1, MainMenuState>),
        )
        //MainMenuState::Exit
        .add_system_set_to_stage(
            CoreStage::PreUpdate,
            SystemSet::on_enter(PreUpdateStageState::MainMenu(Some(MainMenuState::AppExit)))
                .with_system(setup_exit),
        )
        .add_system_set_to_stage(
            CoreStage::Update,
            SystemSet::on_update(UpdateStageState::MainMenu(Some(MainMenuState::AppExit)))
                .with_system(exit_no_button::<1>)
                .with_system(exit_yes_button::<1>)
                .with_system(exit_close_requested),
        );
    }
}

fn button(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut UiColor,
            &Action<for<'a> fn(&'a mut GlobalState)>,
            &HierarchyMark<0>,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut state: ResMut<GlobalState>,
) {
    for (interaction, mut color, func, _) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Clicked => func.run(&mut *state),
            Interaction::Hovered => {
                *color = BUTTON_COLOR_HOVER.into();
            }
            Interaction::None => {
                *color = BUTTON_COLOR_NONE.into();
            }
        }
    }
}

fn setup(mut commands: Commands, state: Res<GlobalState>, asset_server: Res<AssetServer>) {
    // ui camera
    commands
        .spawn_bundle(Camera2dBundle::default())
        .insert(state.mark());

    commands
        .spawn_bundle(create_button())
        .insert(state.mark())
        .insert(Action::<for<'a> fn(&'a mut GlobalState)>::new(
            |g: &mut GlobalState| g.replace(AppState::InGame(None)),
        ))
        .insert(HierarchyMark::<0>)
        .with_children(|parent| {
            parent.spawn_bundle(create_text(
                PLAY_TEXT,
                &asset_server,
                30.0,
                TEXT_COLOR_BRIGHT,
            ));
        });

    commands
        .spawn_bundle(create_button())
        .insert(state.mark())
        .insert(Action::<for<'a> fn(&'a mut GlobalState)>::new(
            |g: &mut GlobalState| g.push(MainMenuState::AppExit),
        ))
        .insert(HierarchyMark::<0>)
        .with_children(|parent| {
            parent.spawn_bundle(create_text(
                EXIT_TEXT,
                &asset_server,
                30.0,
                TEXT_COLOR_BRIGHT,
            ));
        });
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
                .insert(HierarchyMark::<1>)
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
                .insert(HierarchyMark::<1>)
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
