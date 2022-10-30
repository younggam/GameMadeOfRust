pub(crate) mod in_game;
pub(crate) mod main_menu;

use game_made_of_rust::unreachable_release;

use bevy::ecs::system::SystemState;
use bevy::prelude::*;

macro_rules! stage_states {
    ($($stage_name: ident),*; $members:tt) => {
        $(#[derive(Clone, Copy, Eq, PartialEq, Debug, Hash)]
        pub(crate) enum $stage_name $members)*
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
    }
);

pub(crate) trait PopState: Sized {
    fn pop(self) -> Option<Self>;
}

pub(crate) trait PushState {
    fn push(self, parent: &mut AppState);
}

#[derive(Clone, Copy, Eq, PartialEq, Debug, Hash)]
pub(crate) enum MainMenuState {
    Exit,
}

impl PopState for AppState {
    fn pop(self) -> Option<Self> {
        match self {
            AppState::MainMenu(Some(m)) => Some(AppState::MainMenu(m.pop())),
            _ => None,
        }
    }
}

impl PopState for MainMenuState {
    fn pop(self) -> Option<Self> {
        match self {
            MainMenuState::Exit => None,
        }
    }
}

impl PushState for MainMenuState {
    fn push(self, parent: &mut AppState) {
        match *parent {
            AppState::MainMenu(None) => *parent = AppState::MainMenu(Some(self)),
            _ => unreachable_release!("There is no space to push"),
        }
    }
}

pub(crate) mod major_state {
    use crate::states::{AppState, PopState, PushState};
    use bevy::prelude::Component;
    use game_made_of_rust::unreachable_release;
    use std::ops::Deref;

    pub(crate) struct GlobalState(pub AppState, StateChangeWay);

    impl GlobalState {
        pub fn new(initial: AppState) -> Self {
            Self(initial, StateChangeWay::None)
        }

        pub fn should_change(&self) -> bool {
            self.1 != StateChangeWay::None
        }

        pub fn apply_change(&mut self, mut f: impl FnMut(&AppState, &StateChangeWay)) {
            if self.1 == StateChangeWay::None {
                unreachable_release!("No state transition expected");
            }
            f(&self.0, &self.1);
            self.1 = StateChangeWay::None;
        }

        pub fn eq_major(&self, other: &AppState) -> bool {
            match (self.0, other) {
                (AppState::MainMenu(_), AppState::MainMenu(_))
                | (AppState::InGame, AppState::InGame) => true,
                _ => false,
            }
        }

        pub fn should_clear(&self, other: &StateMark) -> bool {
            match (self.0, other.0) {
                (AppState::MainMenu(om0), AppState::MainMenu(om1)) => match (om0, om1) {
                    (Some(m0), Some(m1)) => m0 != m1,
                    (None, Some(_)) => true,
                    _ => false,
                },
                (AppState::InGame, AppState::InGame) => false,
                _ => true,
            }
        }

        pub fn replace(&mut self, to: AppState) {
            if self.eq_major(&to) {
                unreachable_release!("Already in that major state");
            } else if self.1 != StateChangeWay::None {
                unreachable_release!("Already in state transition");
            }
            self.0 = to;
            self.1 = StateChangeWay::Replace;
        }

        pub fn push<Child: PushState>(&mut self, child: Child) {
            if self.1 != StateChangeWay::None {
                unreachable_release!("Already in state transition");
            }
            child.push(&mut self.0);
            self.1 = StateChangeWay::Push;
        }

        pub fn pop(&mut self) {
            if self.1 != StateChangeWay::None {
                unreachable_release!("Already in state transition");
            }
            self.0 = match self.0.pop() {
                Some(a) => a,
                _ => unreachable_release!("Already in state transition"),
            };
            self.1 = StateChangeWay::Pop;
        }
    }

    impl Deref for GlobalState {
        type Target = AppState;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    #[derive(Clone, Copy, Eq, PartialEq, Debug)]
    pub(crate) enum StateChangeWay {
        None,
        Replace,
        Push,
        Pop,
    }

    #[derive(Component)]
    pub(crate) struct StateMark(AppState);

    impl StateMark {
        pub fn new(permanent: AppState) -> Self {
            Self(permanent)
        }
    }
}
use major_state::*;

pub(crate) struct StatesPlugin;

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
                        AppState::InGame => {
                            first.replace(FirstStageState::InGame).unwrap();
                            pre_update.replace(PreUpdateStageState::InGame).unwrap();
                            update.replace(UpdateStageState::InGame).unwrap();
                            post_update.replace(PostUpdateStageState::InGame).unwrap();
                            last.replace(LastStageState::InGame).unwrap();
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
                        _ => unreachable_release!("State is interrupted"),
                    },
                    StateChangeWay::Pop => match state {
                        AppState::MainMenu(None) => {
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
