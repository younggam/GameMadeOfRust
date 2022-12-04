use crate::{asset::Fonts, func::*, states::*, ui::*};

use bevy::prelude::*;

pub struct MainMenuPlugin;

///Batch setup for Main menu.
impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set_to_stage(
            CoreStage::PreUpdate,
            SystemSet::on_enter(PreUpdateStageState::MainMenu).with_system(setup),
        )
        .add_system_set_to_stage(
            CoreStage::Update,
            SystemSet::on_update(UpdateStageState::MainMenu)
                .with_system(button)
                .with_system(close_requested),
        );
    }
}

///Setup system in Main menu.
fn setup(mut commands: Commands, state: Res<GlobalState>, res: Res<Fonts>) {
    //ui camera
    commands.spawn((Camera2dBundle::default(), state.mark()));
    //play button
    commands
        .spawn((
            create_button(),
            state.mark(),
            Action::<for<'a> fn(&'a mut GlobalState)>::new(|g: &mut GlobalState| {
                g.replace(AppState::InGame)
            }),
            HierarchyMark::<0>,
        ))
        .with_children(|parent| {
            parent.spawn(create_text(PLAY_TEXT, &res, 30.0, TEXT_COLOR_BRIGHT));
        });
    //exit button
    commands
        .spawn((
            create_button(),
            state.mark(),
            Action::<for<'a> fn(&'a mut GlobalState)>::new(|g: &mut GlobalState| g.push_exit()),
            HierarchyMark::<0>,
        ))
        .with_children(|parent| {
            parent.spawn(create_text(EXIT_TEXT, &res, 30.0, TEXT_COLOR_BRIGHT));
        });
}

///Buttons interaction system.
fn button(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
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
