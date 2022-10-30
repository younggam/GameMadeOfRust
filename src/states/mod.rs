pub(crate) mod in_game;
pub(crate) mod main_menu;

use bevy::ecs::system::{Resource, SystemState};
use bevy::prelude::*;
use std::ops::Deref;

macro_rules! stage_states {
    ($($stage_name: ident),*; $members:tt; $t:ident, $partial_eq:tt) => {
        $(#[derive(Clone, Eq, Debug, Hash)]
        pub(crate) enum $stage_name $members impl_trait!($t, $stage_name, $partial_eq);)*
    };
}

macro_rules! impl_trait {
    ($t:tt, $i:tt, $b:tt)=>{
        impl $t for $i $b
    };
}

stage_states!(
    AppState,
    FirstStageState,
    PreUpdateStageState,
    UpdateStageState,
    PostUpdateStageState,
    LastStageState;
    {
        MainMenu(Option<MainMenuState>),
        InGame,
    };
    PartialEq,
    {
        fn eq(&self, other: &Self) -> bool {
            match (self, other) {
                (Self::MainMenu(a), Self::MainMenu(b)) => a == b,
                (Self::InGame, Self::InGame) => true,
                _ => false,
            }
        }
    }
);

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub(crate) enum MainMenuState {
    Exit,
}

#[derive(Component)]
pub(crate) struct GlobalState(AppState);

impl GlobalState {
    pub fn should_shift(&self, to: &Self) -> bool {
        match (&self.0, &to.0) {
            (AppState::MainMenu(_), AppState::MainMenu(_))
            | (AppState::InGame, AppState::InGame) => false,
            _ => true,
        }
    }
}

impl Deref for GlobalState {
    type Target = AppState;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub(crate) struct StatesPlugin;

impl Plugin for StatesPlugin {
    fn build(&self, app: &mut App) {
        let manage_state_system_state = ManageStateSystemState::new(&mut app.world);
        let clear_state_system_state = ClearStateSystemState::new(&mut app.world);
        app.insert_resource(manage_state_system_state)
            .insert_resource(clear_state_system_state)
            .insert_resource(GlobalState(AppState::MainMenu(None)))
            //First
            .add_state_to_stage(CoreStage::First, FirstStageState::MainMenu(None))
            .add_system_to_stage(CoreStage::First, manage_state.exclusive_system().at_start())
            //PreUpdate
            .add_state_to_stage(CoreStage::PreUpdate, PreUpdateStageState::MainMenu(None))
            //Update
            .add_state_to_stage(CoreStage::Update, UpdateStageState::MainMenu(None))
            //PostUpdate
            .add_state_to_stage(CoreStage::PostUpdate, PostUpdateStageState::MainMenu(None))
            //Last
            .add_state_to_stage(CoreStage::Last, LastStageState::MainMenu(None));
    }
}

type ManageStateSystemState<'w> = SystemState<(
    Res<'w, GlobalState>,
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
            match app_state.0 {
                AppState::MainMenu(_) => {
                    first.set(FirstStageState::MainMenu(None)).unwrap();
                    pre_update.set(PreUpdateStageState::MainMenu(None)).unwrap();
                    update.set(UpdateStageState::MainMenu(None)).unwrap();
                    post_update
                        .set(PostUpdateStageState::MainMenu(None))
                        .unwrap();
                    last.set(LastStageState::MainMenu(None)).unwrap();
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
    Query<'w, 's, (Entity, &'w GlobalState)>,
    Res<'w, GlobalState>,
)>;

fn clear_state(world: &mut World) {
    world.resource_scope(|world, mut cached_state: Mut<ClearStateSystemState>| {
        let (mut commands, mut despawn_entities_query, app_state) = cached_state.get_mut(world);
        let app_state = app_state.into_inner();
        for (entity, entity_app_state) in despawn_entities_query.iter_mut() {
            if app_state.should_shift(entity_app_state) {
                commands.entity(entity).despawn_recursive();
            }
        }
        cached_state.apply(world);
    });
}
