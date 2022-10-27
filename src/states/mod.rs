pub(crate) mod in_game;
pub(crate) mod main_menu;

use bevy::ecs::schedule::StateData;
use bevy::ecs::system::Resource;
use bevy::prelude::*;

pub(crate) trait GlobalState: Resource {}

#[derive(Clone, Eq, Debug, Hash)]
pub(crate) enum AppState {
    MainMenu,
    InGame,
}

impl PartialEq for AppState {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::MainMenu, Self::MainMenu) | (Self::InGame, Self::InGame) => true,
            _ => false,
        }
    }
}

impl GlobalState for AppState {}

#[derive(Component)]
pub(crate) struct StateComponent<T: GlobalState>(pub(crate) T);

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub(crate) enum FirstStageState {
    MainMenu,
    InGame,
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub(crate) enum PreUpdateStageState {
    MainMenu,
    InGame,
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub(crate) enum UpdateStageState {
    MainMenu,
    InGame,
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub(crate) enum PostUpdateStageState {
    MainMenu,
    InGame,
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub(crate) enum LastStageState {
    MainMenu,
    InGame,
}

pub(crate) struct StatesPlugin;

impl Plugin for StatesPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AppState::MainMenu)
            //First
            .add_state_to_stage(CoreStage::First, FirstStageState::MainMenu)
            .add_system_to_stage(CoreStage::First, manage_state.exclusive_system().at_start())
            //PreUpdate
            .add_state_to_stage(CoreStage::PreUpdate, PreUpdateStageState::MainMenu)
            //Update
            .add_state_to_stage(CoreStage::Update, UpdateStageState::MainMenu)
            //PostUpdate
            .add_state_to_stage(CoreStage::PostUpdate, PostUpdateStageState::MainMenu)
            //Last
            .add_state_to_stage(CoreStage::Last, LastStageState::MainMenu)
            .add_system_to_stage(
                CoreStage::Last,
                clear_state.exclusive_system().before_commands(),
            );
    }
}

fn manage_state(
    state: Res<AppState>,
    first: ResMut<FirstStageState>,
    pre_update: ResMut<PreUpdateStageState>,
    update: ResMut<UpdateStageState>,
    post_update: ResMut<PostUpdateStageState>,
    last: ResMut<LastStageState>,
) {
    if state.is_changed() {
        match state.into_inner() {
            AppState::MainMenu => {
                *first.into_inner() = FirstStageState::MainMenu;
                *pre_update.into_inner() = PreUpdateStageState::MainMenu;
                *update.into_inner() = UpdateStageState::MainMenu;
                *post_update.into_inner() = PostUpdateStageState::MainMenu;
                *last.into_inner() = LastStageState::MainMenu;
            }
            AppState::InGame => {
                *first.into_inner() = FirstStageState::InGame;
                *pre_update.into_inner() = PreUpdateStageState::InGame;
                *update.into_inner() = UpdateStageState::InGame;
                *post_update.into_inner() = PostUpdateStageState::InGame;
                *last.into_inner() = LastStageState::InGame;
            }
        };
    }
}

fn clear_state(
    mut commands: Commands,
    mut despawn_entities_query: Query<(Entity, &StateComponent<AppState>)>,
    app_state: Res<State<AppState>>,
) {
    for (entity, entity_app_state) in despawn_entities_query.iter_mut() {
        if *app_state.current() != entity_app_state.0 {
            commands.entity(entity).despawn_recursive();
        }
    }
}
