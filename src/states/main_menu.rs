use crate::states::*;

use game_made_of_rust::func::*;

use bevy::app::AppExit;
use bevy::prelude::*;

const FONT_DIR: &str = "fonts/Schluber.otf";

const PLAY_TEXT: &str = "Play";
const EXIT_TEXT: &str = "Exit";
const ARE_YOU_SURE_TEXT: &str = "Are you sure?";
const YES_TEXT: &str = "Yes";
const NO_TEXT: &str = "No";

const TEXT_COLOR_BRIGHT: Color = Color::YELLOW;
const TEXT_COLOR_DARK: Color = Color::BLACK;

const BUTTON_COLOR_NONE: Color = Color::BLACK;
const BUTTON_COLOR_HOVER: Color = Color::GRAY;

#[derive(Component)]
pub(crate) struct Hierarchy<const N: i32>;

pub(crate) struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        //MainMenu
        app.add_system_set_to_stage(
            CoreStage::PreUpdate,
            SystemSet::on_enter(PreUpdateStageState::MainMenu(None)).with_system(setup),
        )
        .add_system_set_to_stage(
            CoreStage::Update,
            SystemSet::on_update(UpdateStageState::MainMenu(None)).with_system(main_button),
        )
        //MainMenuState::Exit
        .add_system_set_to_stage(
            CoreStage::PreUpdate,
            SystemSet::on_enter(PreUpdateStageState::MainMenu(Some(MainMenuState::Exit)))
                .with_system(setup_exit),
        )
        .add_system_set_to_stage(
            CoreStage::Update,
            SystemSet::on_update(UpdateStageState::MainMenu(Some(MainMenuState::Exit)))
                .with_system(main_exit_no_button)
                .with_system(main_exit_yes_button),
        );
    }
}

fn main_button(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut UiColor,
            &Action<for<'a> fn(&'a mut GlobalState)>,
            &Hierarchy<0>,
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

fn main_exit_no_button(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut UiColor,
            &Action<for<'a> fn(&'a mut GlobalState)>,
            &Hierarchy<1>,
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

fn main_exit_yes_button(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut UiColor,
            &Action<for<'a> fn(&'a mut EventWriter<AppExit>)>,
            &Hierarchy<1>,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut event: EventWriter<AppExit>,
) {
    for (interaction, mut color, func, _) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Clicked => func.run(&mut event),
            Interaction::Hovered => {
                *color = BUTTON_COLOR_HOVER.into();
            }
            Interaction::None => {
                *color = BUTTON_COLOR_NONE.into();
            }
        }
    }
}

fn create_button() -> ButtonBundle {
    ButtonBundle {
        style: Style {
            size: Size::new(Val::Px(150.0), Val::Px(65.0)),
            // center button
            margin: UiRect::all(Val::Auto),
            // horizontally center child text
            justify_content: JustifyContent::Center,
            // vertically center child text
            align_items: AlignItems::Center,
            ..default()
        },
        color: BUTTON_COLOR_NONE.into(),
        ..default()
    }
}

fn create_text(
    text: impl Into<String>,
    asset_server: &AssetServer,
    size: f32,
    color: Color,
) -> TextBundle {
    TextBundle::from_section(
        text,
        TextStyle {
            font: asset_server.load(FONT_DIR),
            font_size: size,
            color,
        },
    )
    .with_style(Style {
        //center button
        margin: UiRect {
            top: Val::Px(size * 0.25),
            ..default()
        },
        ..default()
    })
    .with_text_alignment(TextAlignment::CENTER)
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // ui camera
    commands
        .spawn_bundle(Camera2dBundle::default())
        .insert(StateMark::new(AppState::MainMenu(None)));

    commands
        .spawn_bundle(create_button())
        .insert(StateMark::new(AppState::MainMenu(None)))
        .insert(Action::<for<'a> fn(&'a mut GlobalState)>::new(
            |g: &mut GlobalState| g.replace(AppState::InGame),
        ))
        .insert(Hierarchy::<0>)
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
        .insert(StateMark::new(AppState::MainMenu(None)))
        .insert(Action::<for<'a> fn(&'a mut GlobalState)>::new(
            |g: &mut GlobalState| g.push(MainMenuState::Exit),
        ))
        .insert(Hierarchy::<0>)
        .with_children(|parent| {
            parent.spawn_bundle(create_text(
                EXIT_TEXT,
                &asset_server,
                30.0,
                TEXT_COLOR_BRIGHT,
            ));
        });
}

fn setup_exit(mut commands: Commands, asset_server: Res<AssetServer>) {
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
        .insert(StateMark::new(AppState::MainMenu(Some(
            MainMenuState::Exit,
        ))))
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
                .insert(Hierarchy::<1>)
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
                .insert(Hierarchy::<1>)
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
