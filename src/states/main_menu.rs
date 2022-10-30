use crate::states::*;

use game_made_of_rust::func::*;

use bevy::app::AppExit;
use bevy::prelude::*;

const FONT_DIR: &str = "fonts/Schluber.otf";

const PLAY_TEXT: &str = "Play";
const EXIT_TEXT: &str = "Exit";

const TEXT_COLOR: Color = Color::YELLOW;

const BUTTON_COLOR_NONE: Color = Color::BLACK;
const BUTTON_COLOR_HOVER: Color = Color::GRAY;

pub(crate) struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set_to_stage(
            CoreStage::PreUpdate,
            SystemSet::on_enter(PreUpdateStageState::MainMenu(None)).with_system(setup),
        )
        .add_system_set_to_stage(
            CoreStage::Update,
            SystemSet::on_update(UpdateStageState::MainMenu(None))
                .with_system(play_button)
                .with_system(exit_button),
        );
    }
}

fn play_button(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut UiColor,
            &Action<for<'a> fn(&'a mut GlobalState)>,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut state: ResMut<GlobalState>,
) {
    for (interaction, mut color, func) in interaction_query.iter_mut() {
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

fn exit_button(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut UiColor,
            &Action<for<'a> fn(&'a mut EventWriter<AppExit>)>,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut event: EventWriter<AppExit>,
) {
    for (interaction, mut color, func) in interaction_query.iter_mut() {
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
        .insert(GlobalState(AppState::MainMenu(None)));

    commands
        .spawn_bundle(create_button())
        .insert(GlobalState(AppState::MainMenu(None)))
        .insert(Action::<for<'a> fn(&'a mut GlobalState)>::new(
            |a: &mut GlobalState| a.0 = AppState::InGame,
        ))
        .with_children(|parent| {
            parent.spawn_bundle(create_text(PLAY_TEXT, &asset_server));
        });

    commands
        .spawn_bundle(create_button())
        .insert(GlobalState(AppState::MainMenu(None)))
        .insert(Action::<for<'a> fn(&'a mut EventWriter<AppExit>)>::new(
            |a: &mut EventWriter<AppExit>| a.send(AppExit),
        ))
        .with_children(|parent| {
            parent.spawn_bundle(create_text(EXIT_TEXT, &asset_server));
        });
}
