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

const TEXT_COLOR: Color = Color::YELLOW;

const BUTTON_COLOR_NONE: Color = Color::BLACK;
const BUTTON_COLOR_HOVER: Color = Color::GRAY;

#[derive(Component)]
pub(crate) struct Hierarchy(i32);

impl PartialEq<i32> for Hierarchy {
    fn eq(&self, other: &i32) -> bool {
        self.0 == *other
    }
}

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
            &Hierarchy,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut state: ResMut<GlobalState>,
) {
    for (interaction, mut color, func, hierarchy) in interaction_query.iter_mut() {
        if *hierarchy != 0 {
            continue;
        }
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
            &Hierarchy,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut state: ResMut<GlobalState>,
) {
    for (interaction, mut color, func, hierarchy) in interaction_query.iter_mut() {
        if *hierarchy != 1 {
            continue;
        }
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
            &Hierarchy,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut event: EventWriter<AppExit>,
) {
    for (interaction, mut color, func, hierarchy) in interaction_query.iter_mut() {
        if *hierarchy != 1 {
            continue;
        }
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

fn create_text(text: impl Into<String>, asset_server: &AssetServer) -> TextBundle {
    TextBundle::from_section(
        text,
        TextStyle {
            font: asset_server.load(FONT_DIR),
            font_size: 30.0,
            color: TEXT_COLOR,
        },
    )
    .with_style(Style {
        // center button
        margin: UiRect {
            top: Val::Px(7.5),
            ..default()
        },
        ..default()
    })
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
        .insert(Hierarchy(0))
        .with_children(|parent| {
            parent.spawn_bundle(create_text(PLAY_TEXT, &asset_server));
        });

    commands
        .spawn_bundle(create_button())
        .insert(StateMark::new(AppState::MainMenu(None)))
        .insert(Action::<for<'a> fn(&'a mut GlobalState)>::new(
            |g: &mut GlobalState| g.push(MainMenuState::Exit),
        ))
        .insert(Hierarchy(0))
        .with_children(|parent| {
            parent.spawn_bundle(create_text(EXIT_TEXT, &asset_server));
        });
}

fn setup_exit(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(NodeBundle {
            transform: Transform {
                translation: Vec3::new(50.0, 50.0, 0.0),
                ..default()
            },
            style: Style {
                margin: UiRect {
                    top: Val::Px(7.5),
                    ..default()
                },
                ..default()
            },
            ..default()
        })
        .insert(StateMark::new(AppState::MainMenu(Some(
            MainMenuState::Exit,
        ))))
        .with_children(|parent| {
            parent.spawn_bundle(create_text(ARE_YOU_SURE_TEXT, &asset_server));

            parent
                .spawn_bundle(create_button())
                .insert(Action::<for<'a> fn(&'a mut EventWriter<AppExit>)>::new(
                    |e: &mut EventWriter<AppExit>| e.send(AppExit),
                ))
                .insert(Hierarchy(1))
                .with_children(|parent| {
                    parent.spawn_bundle(create_text(YES_TEXT, &asset_server));
                });

            parent
                .spawn_bundle(create_button())
                .insert(Action::<for<'a> fn(&'a mut GlobalState)>::new(
                    |g: &mut GlobalState| g.pop(),
                ))
                .insert(Hierarchy(1))
                .with_children(|parent| {
                    parent.spawn_bundle(create_text(NO_TEXT, &asset_server));
                });
        });
}
