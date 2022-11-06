use crate::{func::Action, states::*, Fonts};

use crate::consts::FONT_0;
use bevy::{app::AppExit, input::Input, prelude::*, window::WindowCloseRequested};

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

#[derive(Component)]
pub struct AppExitMark;

pub fn close_requested(
    closed: EventReader<WindowCloseRequested>,
    mut state: ResMut<GlobalState>,
    input: Res<Input<KeyCode>>,
) {
    if !closed.is_empty() || input.just_pressed(KeyCode::Escape) {
        state.push_exit()
    }
}

pub fn exit_close_requested(
    closed: EventReader<WindowCloseRequested>,
    mut event: EventWriter<AppExit>,
) {
    if !closed.is_empty() {
        event.send(AppExit)
    }
}

pub fn exit_esc(mut state: ResMut<GlobalState>, input: Res<Input<KeyCode>>) {
    if input.just_pressed(KeyCode::Escape) {
        state.pop_exit();
    }
}

pub fn exit_no_button(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut UiColor,
            &Action<for<'a> fn(&'a mut GlobalState)>,
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
                *color = BUTTON_COLOR_HOVER.into();
            }
            Interaction::None => {
                *color = BUTTON_COLOR_NONE.into();
            }
        }
    }
}

pub fn exit_yes_button(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut UiColor,
            &Action<for<'a> fn(&'a mut EventWriter<AppExit>)>,
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

pub fn create_text2(
    text: impl Into<String>,
    res: &Res<Fonts>,
    size: f32,
    color: Color,
) -> TextBundle {
    TextBundle::from_section(
        text,
        TextStyle {
            font: (*res.get(FONT_0).unwrap()).clone(),
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

pub fn setup_exit(
    mut commands: Commands,
    state: Res<GlobalState>,
    fonts: Res<Fonts>,
) {
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
                    parent.spawn_bundle(create_text2(
                        ARE_YOU_SURE_TEXT,
                        &fonts,
                        30.0,
                        TEXT_COLOR_DARK,
                    ));
                });

            parent
                .spawn_bundle(create_button())
                .insert(Action::<for<'a> fn(&'a mut EventWriter<AppExit>)>::new(
                    |e: &mut EventWriter<AppExit>| e.send(AppExit),
                ))
                .insert(AppExitMark)
                .with_children(|parent| {
                    parent.spawn_bundle(create_text2(YES_TEXT, &fonts, 30.0, TEXT_COLOR_BRIGHT));
                });

            parent
                .spawn_bundle(create_button())
                .insert(Action::<for<'a> fn(&'a mut GlobalState)>::new(
                    |g: &mut GlobalState| g.pop_exit(),
                ))
                .insert(AppExitMark)
                .with_children(|parent| {
                    parent.spawn_bundle(create_text2(NO_TEXT, &fonts, 30.0, TEXT_COLOR_BRIGHT));
                });
        });
}
