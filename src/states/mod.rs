pub(crate) mod in_game;
pub(crate) mod main_menu;

use bevy::ecs::system::{Resource, SystemState};
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
pub(crate) struct StateComponent<S: GlobalState>(pub(crate) S);

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
        let manage_state_system_state = ManageStateSystemState::new(&mut app.world);
        let clear_state_system_state = ClearStateSystemState::new(&mut app.world);
        app.insert_resource(manage_state_system_state)
            .insert_resource(clear_state_system_state)
            .insert_resource(AppState::MainMenu)
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
            .add_state_to_stage(CoreStage::Last, LastStageState::MainMenu);
    }
}

type ManageStateSystemState<'w> = SystemState<(
    Res<'w, AppState>,
    ResMut<'w, State<FirstStageState>>,
    ResMut<'w, State<PreUpdateStageState>>,
    ResMut<'w, State<UpdateStageState>>,
    ResMut<'w, State<PostUpdateStageState>>,
    ResMut<'w, State<LastStageState>>,
)>;

fn manage_state(world: &mut World) {
    world.resource_scope(|world, mut cached_state: Mut<ManageStateSystemState>| {
        let (app_state, mut first, mut pre_update, mut update, mut post_update, mut last) =
            cached_state.get_mut(world);

        if app_state.is_changed() && !app_state.is_added() {
            match *app_state {
                AppState::MainMenu => {
                    first.set(FirstStageState::MainMenu).unwrap();
                    pre_update.set(PreUpdateStageState::MainMenu).unwrap();
                    update.set(UpdateStageState::MainMenu).unwrap();
                    post_update.set(PostUpdateStageState::MainMenu).unwrap();
                    last.set(LastStageState::MainMenu).unwrap();
                }
                AppState::InGame => {
                    first.set(FirstStageState::InGame).unwrap();
                    pre_update.set(PreUpdateStageState::InGame).unwrap();
                    update.set(UpdateStageState::InGame).unwrap();
                    post_update.set(PostUpdateStageState::InGame).unwrap();
                    last.set(LastStageState::InGame).unwrap();
                }
            };
        }
    });

    clear_state(world);
}

type ClearStateSystemState<'w, 's> = SystemState<(
    Commands<'w, 's>,
    Query<'w, 's, (Entity, &'w StateComponent<AppState>)>,
    Res<'w, AppState>,
)>;

fn clear_state(world: &mut World) {
    world.resource_scope(|world, mut cached_state: Mut<ClearStateSystemState>| {
        let (mut commands, mut despawn_entities_query, app_state) = cached_state.get_mut(world);
        let app_state = app_state.into_inner();
        for (entity, entity_app_state) in despawn_entities_query.iter_mut() {
            if *app_state != entity_app_state.0 {
                commands.entity(entity).despawn_recursive();
            }
        }
        cached_state.apply(world);
    });
}