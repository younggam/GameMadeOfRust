use crate::{
    asset::{Fonts, FONT_SCHLUBER},
    func::Action,
    states::*,
};

use bevy::{app::AppExit, input::Input, prelude::*, window::WindowCloseRequested};

pub const PLAY_TEXT: &str = "Play";
pub const EXIT_TEXT: &str = "Exit";
pub const ARE_YOU_SURE_TEXT: &str = "Are you sure?";
pub const YES_TEXT: &str = "Yes";
pub const NO_TEXT: &str = "No";

pub const UI_BACKGROUND_COLOR: BackgroundColor = BackgroundColor(Color::WHITE);

pub const TEXT_COLOR_BRIGHT: Color = Color::YELLOW;
pub const TEXT_COLOR_DARK: Color = Color::BLACK;

pub const BUTTON_COLOR_NONE: BackgroundColor = BackgroundColor(Color::BLACK);
pub const BUTTON_COLOR_HOVER: BackgroundColor = BackgroundColor(Color::GRAY);

///Mark hierarchy info of ui
#[derive(Component)]
pub struct HierarchyMark<const N: u32>;

///Mark ui is for exit.
#[derive(Component)]
pub struct AppExitMark;

///Go to exit state when requested.
pub fn close_requested(
    closed: EventReader<WindowCloseRequested>,
    mut state: ResMut<GlobalState>,
    input: Res<Input<KeyCode>>,
) {
    if !closed.is_empty() || input.just_pressed(KeyCode::Escape) {
        state.push_exit()
    }
}

///Force app exit via close request on exit state.
pub fn exit_close_requested(
    closed: EventReader<WindowCloseRequested>,
    mut event: EventWriter<AppExit>,
) {
    if !closed.is_empty() {
        event.send(AppExit)
    }
}

///Close exit state via esc.
pub fn exit_esc(mut state: ResMut<GlobalState>, input: Res<Input<KeyCode>>) {
    if input.just_pressed(KeyCode::Escape) {
        state.pop_exit();
    }
}

///Interaction with no button of exit popup.
pub fn exit_no_button(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &Action<fn(&mut GlobalState)>,
            &AppExitMark,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut state: ResMut<GlobalState>,
) {
    for (interaction, mut color, func, _) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Clicked => func.run(&mut *state),
            Interaction::Hovered => {
                *color = BUTTON_COLOR_HOVER;
            }
            Interaction::None => {
                *color = BUTTON_COLOR_NONE;
            }
        }
    }
}

///Interaction with yes button of exit popup.
pub fn exit_yes_button(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &Action<fn(&mut EventWriter<AppExit>)>,
            &AppExitMark,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut event: EventWriter<AppExit>,
) {
    for (interaction, mut color, func, _) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Clicked => func.run(&mut event),
            Interaction::Hovered => {
                *color = BUTTON_COLOR_HOVER;
            }
            Interaction::None => {
                *color = BUTTON_COLOR_NONE;
            }
        }
    }
}

///Shortcut to create button.
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
        background_color: BUTTON_COLOR_NONE,
        ..default()
    }
}

///Shortcut to create text.
pub fn create_text(
    text: impl Into<String>,
    fonts: &Res<Fonts>,
    size: f32,
    color: Color,
) -> TextBundle {
    TextBundle::from_section(
        text,
        TextStyle {
            font: fonts[FONT_SCHLUBER].clone(),
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

///Setup exit popup.
pub fn setup_exit(mut commands: Commands, state: Res<GlobalState>, fonts: Res<Fonts>) {
    //Node that represent popup.
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    size: Size::new(Val::Percent(40.0), Val::Percent(24.0)),
                    position_type: PositionType::Absolute,
                    position: UiRect::new(
                        Val::Percent(30.0),
                        Val::Percent(30.0),
                        Val::Percent(38.0),
                        Val::Percent(38.0),
                    ),
                    flex_wrap: FlexWrap::Wrap,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    align_content: AlignContent::SpaceAround,
                    ..default()
                },
                background_color: UI_BACKGROUND_COLOR,
                ..default()
            },
            state.mark(),
        ))
        .with_children(|parent| {
            //Container for text.
            parent
                .spawn(NodeBundle {
                    style: Style {
                        flex_basis: Val::Percent(100.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    ..default()
                })
                //text
                .with_children(|parent| {
                    parent.spawn(create_text(
                        ARE_YOU_SURE_TEXT,
                        &fonts,
                        30.0,
                        TEXT_COLOR_DARK,
                    ));
                });
            //yes button
            parent
                .spawn((
                    create_button(),
                    Action::<for<'a> fn(&'a mut EventWriter<AppExit>)>::new(
                        |e: &mut EventWriter<AppExit>| e.send(AppExit),
                    ),
                    AppExitMark,
                ))
                .with_children(|parent| {
                    parent.spawn(create_text(YES_TEXT, &fonts, 30.0, TEXT_COLOR_BRIGHT));
                });
            //no button
            parent
                .spawn((
                    create_button(),
                    Action::<for<'a> fn(&'a mut GlobalState)>::new(|g: &mut GlobalState| {
                        g.pop_exit()
                    }),
                    AppExitMark,
                ))
                .with_children(|parent| {
                    parent.spawn(create_text(NO_TEXT, &fonts, 30.0, TEXT_COLOR_BRIGHT));
                });
        });
}
