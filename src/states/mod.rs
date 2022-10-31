pub mod in_game;
pub mod main_menu;

use crate::unreachable_release;

use bevy::{ecs::system::SystemState, prelude::*};

macro_rules! stage_states {
    ($($stage_name: ident),*; $members:tt) => {
        $(#[derive(Clone, Copy, Eq, PartialEq, Debug, Hash)]
        pub enum $stage_name $members)*
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
        InGame(Option<InGameState>),
    }
);

pub trait PopState: Sized {
    fn pop(self) -> Option<Self>;
}

pub trait PushState {
    const APP_EXIT: Self;

    fn push(self, parent: &mut AppState);
}

impl PopState for AppState {
    fn pop(self) -> Option<Self> {
        match self {
            AppState::MainMenu(Some(m)) => Some(AppState::MainMenu(m.pop())),
            AppState::InGame(Some(i)) => Some(AppState::InGame(i.pop())),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Debug, Hash)]
pub enum MainMenuState {
    AppExit,
}

impl PopState for MainMenuState {
    fn pop(self) -> Option<Self> {
        match self {
            MainMenuState::AppExit => None,
        }
    }
}

impl PushState for MainMenuState {
    const APP_EXIT: Self = MainMenuState::AppExit;

    fn push(self, parent: &mut AppState) {
        match *parent {
            AppState::MainMenu(None) => *parent = AppState::MainMenu(Some(self)),
            _ => unreachable_release!("There is no space to push"),
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Debug, Hash)]
pub enum InGameState {
    AppExit,
}

impl PopState for InGameState {
    fn pop(self) -> Option<Self> {
        match self {
            InGameState::AppExit => None,
        }
    }
}

impl PushState for InGameState {
    const APP_EXIT: Self = InGameState::AppExit;

    fn push(self, parent: &mut AppState) {
        match *parent {
            AppState::InGame(None) => *parent = AppState::InGame(Some(self)),
            _ => unreachable_release!("There is no space to push"),
        }
    }
}

mod global {
    use crate::{states::*, unreachable_release};

    use bevy::prelude::Component;

    #[derive(Clone, Copy, Debug, Ord, PartialOrd, Eq, PartialEq)]
    pub struct Hierarchy(u32);

    impl Hierarchy {
        pub const fn new<const N: u32>() -> Self {
            Self(N)
        }

        fn reset(&mut self) {
            self.0 = 0
        }

        fn increment(&mut self) {
            self.0 += 1
        }

        fn decrement(&mut self) {
            self.0 -= 1
        }

        fn is<const N: u32>(&self) -> bool {
            self.0 == N
        }
    }

    pub struct GlobalState(pub AppState, Hierarchy, StateChangeWay);

    impl GlobalState {
        pub fn new(initial: AppState) -> Self {
            Self(initial, Hierarchy::new::<0>(), StateChangeWay::None)
        }

        pub fn is_hierarchy<const N: u32>(&self) -> bool {
            self.1.is::<N>()
        }

        pub fn mark(&self) -> StateMark {
            StateMark(self.0, self.1)
        }

        pub fn should_change(&self) -> bool {
            self.2 != StateChangeWay::None
        }

        pub fn apply_change(&mut self, mut f: impl FnMut(&AppState, &StateChangeWay)) {
            if self.2 == StateChangeWay::None {
                unreachable_release!("No state transition expected");
            }
            f(&self.0, &self.2);
            self.2 = StateChangeWay::None;
        }

        pub fn eq_major(&self, other: &AppState) -> bool {
            match (self.0, other) {
                (AppState::MainMenu(_), AppState::MainMenu(_))
                | (AppState::InGame(_), AppState::InGame(_)) => true,
                _ => false,
            }
        }

        pub fn should_clear(&self, other: &StateMark) -> bool {
            self.1 < other.1 || (self.1 == other.1 && self.0 != other.0)
        }

        pub fn replace(&mut self, to: AppState) {
            if self.eq_major(&to) {
                unreachable_release!("Already in that major state");
            } else if self.2 != StateChangeWay::None {
                unreachable_release!("Already in state transition");
            }
            self.0 = to;
            self.1.reset();
            self.2 = StateChangeWay::Replace;
        }

        pub fn push<Child: PushState>(&mut self, child: Child) {
            if self.2 != StateChangeWay::None {
                unreachable_release!("Already in state transition");
            }
            child.push(&mut self.0);
            self.1.increment();
            self.2 = StateChangeWay::Push;
        }

        pub fn pop(&mut self) {
            if self.2 != StateChangeWay::None {
                unreachable_release!("Already in state transition");
            }
            self.0 = match self.0.pop() {
                Some(a) => a,
                _ => unreachable_release!("Already in state transition"),
            };
            self.1.decrement();
            self.2 = StateChangeWay::Pop;
        }
    }

    #[derive(Clone, Copy, Eq, PartialEq, Debug)]
    pub enum StateChangeWay {
        None,
        Replace,
        Push,
        Pop,
    }

    #[derive(Component)]
    pub struct StateMark(AppState, Hierarchy);
}
pub use global::*;

pub struct StatesPlugin;

impl Plugin for StatesPlugin {
    fn build(&self, app: &mut App) {
        let manage_state_system_state = ManageStateSystemState::new(&mut app.world);
        let clear_state_system_state = ClearStateSystemState::new(&mut app.world);
        app.insert_resource(manage_state_system_state)
            .insert_resource(clear_state_system_state)
            .insert_resource(GlobalState::new(AppState::MainMenu(None)))
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
    ResMut<'w, GlobalState>,
    ResMut<'w, State<FirstStageState>>,
    ResMut<'w, State<PreUpdateStageState>>,
    ResMut<'w, State<UpdateStageState>>,
    ResMut<'w, State<PostUpdateStageState>>,
    ResMut<'w, State<LastStageState>>,
)>;

fn manage_state(world: &mut World) {
    world.resource_scope(|world, mut cached_state: Mut<ManageStateSystemState>| {
        let (mut app_state, mut first, mut pre_update, mut update, mut post_update, mut last) =
            cached_state.get_mut(world);
        if app_state.should_change() {
            app_state.apply_change(|state, change_way| {
                match change_way {
                    StateChangeWay::Replace => match *state {
                        AppState::MainMenu(m) => {
                            first.replace(FirstStageState::MainMenu(m)).unwrap();
                            pre_update
                                .replace(PreUpdateStageState::MainMenu(m))
                                .unwrap();
                            update.replace(UpdateStageState::MainMenu(m)).unwrap();
                            post_update
                                .replace(PostUpdateStageState::MainMenu(m))
                                .unwrap();
                            last.replace(LastStageState::MainMenu(m)).unwrap();
                        }
                        AppState::InGame(i) => {
                            first.replace(FirstStageState::InGame(i)).unwrap();
                            pre_update.replace(PreUpdateStageState::InGame(i)).unwrap();
                            update.replace(UpdateStageState::InGame(i)).unwrap();
                            post_update
                                .replace(PostUpdateStageState::InGame(i))
                                .unwrap();
                            last.replace(LastStageState::InGame(i)).unwrap();
                        }
                    },
                    StateChangeWay::Push => match *state {
                        AppState::MainMenu(Some(m)) => {
                            first.push(FirstStageState::MainMenu(Some(m))).unwrap();
                            pre_update
                                .push(PreUpdateStageState::MainMenu(Some(m)))
                                .unwrap();
                            update.push(UpdateStageState::MainMenu(Some(m))).unwrap();
                            post_update
                                .push(PostUpdateStageState::MainMenu(Some(m)))
                                .unwrap();
                            last.push(LastStageState::MainMenu(Some(m))).unwrap();
                        }
                        AppState::InGame(Some(i)) => {
                            first.push(FirstStageState::InGame(Some(i))).unwrap();
                            pre_update
                                .push(PreUpdateStageState::InGame(Some(i)))
                                .unwrap();
                            update.push(UpdateStageState::InGame(Some(i))).unwrap();
                            post_update
                                .push(PostUpdateStageState::InGame(Some(i)))
                                .unwrap();
                            last.push(LastStageState::InGame(Some(i))).unwrap();
                        }
                        _ => unreachable_release!("State is interrupted"),
                    },
                    StateChangeWay::Pop => match state {
                        AppState::MainMenu(None) | AppState::InGame(None) => {
                            first.pop().unwrap();
                            pre_update.pop().unwrap();
                            update.pop().unwrap();
                            post_update.pop().unwrap();
                            last.pop().unwrap();
                        }
                        _ => unreachable_release!("State is interrupted"),
                    },
                    _ => unreachable_release!("State is interrupted"),
                };
            });

            clear_state(world);
        }
    });
}

type ClearStateSystemState<'w, 's> = SystemState<(
    Commands<'w, 's>,
    Query<'w, 's, (Entity, &'w StateMark)>,
    Res<'w, GlobalState>,
)>;

fn clear_state(world: &mut World) {
    world.resource_scope(|world, mut cached_state: Mut<ClearStateSystemState>| {
        let (mut commands, mut despawn_entities_query, app_state) = cached_state.get_mut(world);
        let app_state = app_state.into_inner();
        for (entity, state_mark) in despawn_entities_query.iter_mut() {
            if app_state.should_clear(state_mark) {
                commands.entity(entity).despawn_recursive();
            }
        }
        cached_state.apply(world);
    });
}
