pub(crate) mod in_game;
pub(crate) mod main_menu;

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

fn manage_state(world: &mut World) {
    if world.is_resource_changed::<AppState>() && !world.is_resource_added::<AppState>() {
        match world.resource::<AppState>() {
            AppState::MainMenu => {
                world
                    .resource_mut::<State<FirstStageState>>()
                    .overwrite_set(FirstStageState::MainMenu)
                    .unwrap();
                world
                    .resource_mut::<State<PreUpdateStageState>>()
                    .overwrite_set(PreUpdateStageState::MainMenu)
                    .unwrap();
                world
                    .resource_mut::<State<UpdateStageState>>()
                    .overwrite_set(UpdateStageState::MainMenu)
                    .unwrap();
                world
                    .resource_mut::<State<PostUpdateStageState>>()
                    .overwrite_set(PostUpdateStageState::MainMenu)
                    .unwrap();
                world
                    .resource_mut::<State<LastStageState>>()
                    .overwrite_set(LastStageState::MainMenu)
                    .unwrap();
            }
            AppState::InGame => {
                world
                    .resource_mut::<State<FirstStageState>>()
                    .overwrite_set(FirstStageState::InGame)
                    .unwrap();
                world
                    .resource_mut::<State<PreUpdateStageState>>()
                    .overwrite_set(PreUpdateStageState::InGame)
                    .unwrap();
                world
                    .resource_mut::<State<UpdateStageState>>()
                    .overwrite_set(UpdateStageState::InGame)
                    .unwrap();
                world
                    .resource_mut::<State<PostUpdateStageState>>()
                    .overwrite_set(PostUpdateStageState::InGame)
                    .unwrap();
                world
                    .resource_mut::<State<LastStageState>>()
                    .overwrite_set(LastStageState::InGame)
                    .unwrap();
            }
        };
    }
}

fn clear_state(
    mut commands: Commands,
    mut despawn_entities_query: Query<(Entity, &StateComponent<AppState>)>,
    app_state: Res<AppState>,
) {
    let app_state = app_state.into_inner();
    for (entity, entity_app_state) in despawn_entities_query.iter_mut() {
        if *app_state != entity_app_state.0 {
            commands.entity(entity).despawn_recursive();
        }
    }
}
