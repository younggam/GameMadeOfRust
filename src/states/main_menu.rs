use crate::{func::*, states::*, ui::*, Fonts};

use bevy::prelude::*;

pub struct MainMenuPlugin;

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

fn button(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut UiColor,
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

fn setup(
    mut commands: Commands,
    state: Res<GlobalState>,
    res: Res<Fonts>,
    assets: Res<Assets<Font>>,
) {
    // ui camera
    commands
        .spawn_bundle(Camera2dBundle::default())
        .insert(state.mark());

    commands
        .spawn_bundle(create_button())
        .insert(state.mark())
        .insert(Action::<for<'a> fn(&'a mut GlobalState)>::new(
            |g: &mut GlobalState| g.replace(AppState::InGame),
        ))
        .insert(HierarchyMark::<0>)
        .with_children(|parent| {
            parent.spawn_bundle(create_text2(
                PLAY_TEXT,
                &res,
                &assets,
                30.0,
                TEXT_COLOR_BRIGHT,
            ));
        });

    commands
        .spawn_bundle(create_button())
        .insert(state.mark())
        .insert(Action::<for<'a> fn(&'a mut GlobalState)>::new(
            |g: &mut GlobalState| g.push_exit(),
        ))
        .insert(HierarchyMark::<0>)
        .with_children(|parent| {
            parent.spawn_bundle(create_text2(
                EXIT_TEXT,
                &res,
                &assets,
                30.0,
                TEXT_COLOR_BRIGHT,
            ));
        });
}
