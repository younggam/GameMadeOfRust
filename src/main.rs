use bevy::prelude::*;

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum AppState {
    MainMenu,
    InGame,
}

#[derive(Component)]
struct AppStateComponent(AppState);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_system_to_stage(CoreStage::First, || println!("first"))
        .add_startup_system_to_stage(StartupStage::PreStartup, || println!("pre start up"))
        //Main Menu
        .add_state(AppState::MainMenu)
        .add_system_set(SystemSet::on_enter(AppState::MainMenu).with_system(setup))
        .add_system_set(
            SystemSet::on_update(AppState::MainMenu)
                .with_system(button_system)
                .with_system(|time: Res<Time>| {
                    println!("update menu {:?}", time.time_since_startup())
                })
                .with_system(enter_game),
        )
        .add_system_set(
            SystemSet::on_exit(AppState::MainMenu)
                .with_system(|time: Res<Time>| println!("bye menu {:?}", time.time_since_startup()))
                .with_system(clear_state_system),
        )
        //In Game
        .add_system_set(
            SystemSet::on_enter(AppState::InGame)
                .with_system(|time: Res<Time>| println!("hi game {:?}", time.time_since_startup())),
        )
        .add_system_set(
            SystemSet::on_update(AppState::InGame)
                .with_system(|time: Res<Time>| {
                    println!("update game {:?}", time.time_since_startup())
                })
                .with_system(enter_menu),
        )
        .add_system_set(
            SystemSet::on_exit(AppState::InGame)
                .with_system(|time: Res<Time>| println!("bye game {:?}", time.time_since_startup()))
                .with_system(clear_state_system),
        )
        .run();
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
        .insert(AppStateComponent(AppState::MainMenu));
    commands.spawn_bundle(ButtonBundle {
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
        .insert(AppStateComponent(AppState::MainMenu))
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
    commands.spawn_bundle(ButtonBundle {
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
        .insert(AppStateComponent(AppState::MainMenu))
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

fn enter_game(input: Res<Input<KeyCode>>, mut app_state: ResMut<State<AppState>>) {
    if !input.pressed(KeyCode::LShift) {
        return;
    }
    app_state.set(AppState::InGame).unwrap();
    // ^ this can fail if we are already in the target state
    // or if another state change is already queued
}

fn enter_menu(input: Res<Input<KeyCode>>, mut app_state: ResMut<State<AppState>>) {
    if !input.pressed(KeyCode::RShift) {
        return;
    }
    app_state.set(AppState::MainMenu).unwrap();
    // ^ this can fail if we are already in the target state
    // or if another state change is already queued
}

fn clear_state_system(
    mut commands: Commands,
    mut despawn_entities_query: Query<(Entity, &AppStateComponent)>,
    app_state: Res<State<AppState>>,
) {
    for (entity, entity_app_state) in despawn_entities_query.iter_mut() {
        if *app_state.current() == entity_app_state.0 {
            commands.entity(entity).despawn_recursive();
        }
    }
}
