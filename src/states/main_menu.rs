use crate::states::*;

use bevy::prelude::*;

pub(crate) struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set_to_stage(
            CoreStage::PreUpdate,
            SystemSet::on_enter(PreUpdateStageState::MainMenu).with_system(setup),
        )
        .add_system_set_to_stage(
            CoreStage::Update,
            SystemSet::on_update(UpdateStageState::MainMenu).with_system(button_system),
        );
    }
}

fn button_system(
    mut interaction_query: Query<
        (&Interaction, &mut UiColor, &Children),
        (Changed<Interaction>, With<Button>),
    >,
    mut text_query: Query<&mut Text>,
) {
    for (interaction, mut color, children) in &mut interaction_query {
        let mut text = text_query.get_mut(children[0]).unwrap();
        match *interaction {
            Interaction::Clicked => {
                text.sections[0].value = "Press".to_string();
                *color = Color::YELLOW_GREEN.into();
            }
            Interaction::Hovered => {
                text.sections[0].value = "Hover".to_string();
                *color = Color::YELLOW.into();
            }
            Interaction::None => {
                text.sections[0].value = "Button".to_string();
                *color = Color::WHITE.into();
            }
        }
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // ui camera
    commands
        .spawn_bundle(Camera2dBundle::default())
        .insert(StateComponent(AppState::MainMenu));
    commands
        .spawn_bundle(ButtonBundle {
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
            color: Color::WHITE.into(),
            ..default()
        })
        .insert(StateComponent(AppState::MainMenu))
        .with_children(|parent| {
            parent.spawn_bundle(
                TextBundle::from_section(
                    "Button",
                    TextStyle {
                        font: asset_server.load("fonts/Schluber.otf"),
                        font_size: 30.0,
                        color: Color::BLACK,
                    },
                )
                .with_style(Style {
                    // center button
                    margin: UiRect {
                        top: Val::Px(7.5),
                        ..default()
                    },
                    ..default()
                }),
            );
        });
    commands
        .spawn_bundle(ButtonBundle {
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
            color: Color::WHITE.into(),
            ..default()
        })
        .insert(StateComponent(AppState::MainMenu))
        .with_children(|parent| {
            parent.spawn_bundle(
                TextBundle::from_section(
                    "Button",
                    TextStyle {
                        font: asset_server.load("fonts/Schluber.otf"),
                        font_size: 30.0,
                        color: Color::BLACK,
                    },
                )
                .with_style(Style {
                    // center button
                    margin: UiRect {
                        top: Val::Px(7.5),
                        ..default()
                    },
                    ..default()
                }),
            );
        });
}
