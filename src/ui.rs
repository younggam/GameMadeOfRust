use crate::{func::Action, states::*};

use bevy::{
    app::AppExit, asset::AssetServer, input::Input, prelude::*, window::WindowCloseRequested,
};

pub const FONT_DIR: &str = "fonts/Schluber.otf";

pub const PLAY_TEXT: &str = "Play";
pub const EXIT_TEXT: &str = "Exit";
pub const ARE_YOU_SURE_TEXT: &str = "Are you sure?";
pub const YES_TEXT: &str = "Yes";
pub const NO_TEXT: &str = "No";

pub const TEXT_COLOR_BRIGHT: Color = Color::YELLOW;
pub const TEXT_COLOR_DARK: Color = Color::BLACK;

pub const BUTTON_COLOR_NONE: Color = Color::BLACK;
pub const BUTTON_COLOR_HOVER: Color = Color::GRAY;

#[derive(Component)]
pub struct HierarchyMark<const N: u32>;

pub fn close_requested<const N: u32, S: PushState>(
    closed: EventReader<WindowCloseRequested>,
    mut state: ResMut<GlobalState>,
    input: Res<Input<KeyCode>>,
) {
    if (!closed.is_empty() || input.just_pressed(KeyCode::Escape)) && state.is_hierarchy::<N>() {
        state.push(S::APP_EXIT)
    }
}

pub fn exit_close_requested(
    closed: EventReader<WindowCloseRequested>,
    mut event: EventWriter<AppExit>,
    input: Res<Input<KeyCode>>,
) {
    if !closed.is_empty() || input.just_pressed(KeyCode::Escape) {
        event.send(AppExit)
    }
}

pub fn exit_no_button<const N: u32>(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut UiColor,
            &Action<for<'a> fn(&'a mut GlobalState)>,
            &HierarchyMark<N>,
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

pub fn exit_yes_button<const N: u32>(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut UiColor,
            &Action<for<'a> fn(&'a mut EventWriter<AppExit>)>,
            &HierarchyMark<N>,
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

pub fn create_button() -> ButtonBundle {
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

pub fn create_text(
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
