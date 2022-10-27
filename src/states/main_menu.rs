use crate::states::*;

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
            SystemSet::on_enter(PreUpdateStageState::MainMenu).with_system(setup),
        )
        .add_system_set_to_stage(
            CoreStage::Update,
            SystemSet::on_update(UpdateStageState::MainMenu).with_system(interact_buttons),
        );
    }
}

#[derive(Component)]
pub(crate) struct Action<F>(F);

fn interact_buttons(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut UiColor,
            &Action<for<'a> fn(&'a mut AppState)>,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut state: ResMut<AppState>,
) {
    for (interaction, mut color, func) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Clicked => func.0(&mut *state),
            Interaction::Hovered => {
                *color = BUTTON_COLOR_HOVER.into();
            }
            Interaction::None => {
                *color = BUTTON_COLOR_NONE.into();
            }
        }
    }
}

fn button() -> ButtonBundle {
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

fn text(text: impl Into<String>, asset_server: &AssetServer) -> TextBundle {
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

fn but(a: &mut AppState) {
    *a = AppState::InGame;
    *a = AppState::InGame;
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // ui camera
    commands
        .spawn_bundle(Camera2dBundle::default())
        .insert(StateComponent(AppState::MainMenu));

    commands
        .spawn_bundle(button())
        .insert(StateComponent(AppState::MainMenu))
        .insert(Action::<for<'a> fn(&'a mut AppState)>(but))
        .with_children(|parent| {
            parent.spawn_bundle(text(PLAY_TEXT, &asset_server));
        });

    commands
        .spawn_bundle(button())
        .insert(StateComponent(AppState::MainMenu))
        .with_children(|parent| {
            parent.spawn_bundle(text(EXIT_TEXT, &asset_server));
        });
}
