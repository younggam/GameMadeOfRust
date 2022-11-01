pub mod in_game;
pub mod main_menu;

use crate::unreachable_release;

use bevy::{ecs::system::SystemState, prelude::*};

macro_rules! stage_states {
    ($global_name:ident; $($stage_name:ident),*; $common:tt $local:tt) => {
        #[derive(Clone, Copy, Eq, PartialEq, Debug, Hash)]
        pub enum $global_name $common
        $(
            stage_states!(@ $stage_name $common $local);
        )*
    };
    (@ $stage_name:ident {$($common:ident),*} {$($local:ident),*}) => {
        #[derive(Clone, Copy, Eq, PartialEq, Debug, Hash)]
        pub enum $stage_name {
            $($common,)*
            $($local),*
        }
    }
}

stage_states!(
    AppState;
    FirstStageState,
    PreUpdateStageState,
    UpdateStageState,
    PostUpdateStageState,
    LastStageState;
    {
        MainMenu,
        InGame
    }
    {
        AppExit
    }
);

pub trait PopState: Sized {
    fn pop(self) -> Option<Self>;
}

pub trait PushState {
    fn push(self, parent: &mut AppState);
}

impl PopState for AppState {
    fn pop(self) -> Option<Self> {
        // match self {
        //     AppState::MainMenu(Some(m)) => Some(AppState::MainMenu(m.pop())),
        //     AppState::InGame(Some(i)) => Some(AppState::InGame(i.pop())),
        //     _ => None,
        // }
        None
    }
}

// #[derive(Clone, Copy, Eq, PartialEq, Debug, Hash)]
// pub enum MainMenuState {
//     AppExit,
// }
//
// impl PopState for MainMenuState {
//     fn pop(self) -> Option<Self> {
//         match self {
//             MainMenuState::AppExit => None,
//         }
//     }
// }
//
// impl PushState for MainMenuState {//
//     fn push(self, parent: &mut AppState) {
//         match *parent {
//             AppState::MainMenu(None) => *parent = AppState::MainMenu(Some(self)),
//             _ => unreachable_release!("There is no space to push"),
//         }
//     }
// }
//
// #[derive(Clone, Copy, Eq, PartialEq, Debug, Hash)]
// pub enum InGameState {
//     AppExit,
// }
//
// impl PopState for InGameState {
//     fn pop(self) -> Option<Self> {
//         match self {
//             InGameState::AppExit => None,
//         }
//     }
// }
//
// impl PushState for InGameState {
//     fn push(self, parent: &mut AppState) {
//         match *parent {
//             AppState::InGame(None) => *parent = AppState::InGame(Some(self)),
//             _ => unreachable_release!("There is no space to push"),
//         }
//     }
// }

mod global {
    use crate::{states::*, unreachable_release};

    use bevy::prelude::Component;

    #[derive(Clone, Copy, Ord, PartialOrd, Debug, Eq, PartialEq)]
    pub struct Hierarchy {
        value: u32,
        exit: bool,
    }

    impl Hierarchy {
        pub const fn new<const N: u32>() -> Self {
            Self {
                value: N,
                exit: false,
            }
        }

        fn reset(&mut self) {
            self.value = 0;
            self.exit = false;
        }

        fn increment(&mut self) {
            self.value += 1
        }

        fn decrement(&mut self) {
            self.value -= 1
        }

        fn set_exit(&mut self, exit: bool) {
            self.exit = exit
        }

        fn is_exit(&self) -> bool {
            self.exit
        }
    }

    pub struct GlobalState {
        app_state: AppState,
        hierarchy: Hierarchy,
        state_change_way: StateChangeWay,
    }

    impl GlobalState {
        pub fn new(initial: AppState) -> Self {
            Self {
                app_state: initial,
                hierarchy: Hierarchy::new::<0>(),
                state_change_way: StateChangeWay::None,
            }
        }

        pub fn is_exit(&self) -> bool {
            self.hierarchy.is_exit()
        }

        pub fn mark(&self) -> StateMark {
            StateMark(self.app_state, self.hierarchy)
        }

        pub fn should_change(&self) -> bool {
            self.state_change_way != StateChangeWay::None
        }

        pub fn propagate_change(&mut self, mut f: impl FnMut(&AppState, bool, &StateChangeWay)) {
            if self.state_change_way == StateChangeWay::None {
                unreachable_release!("No state transition expected");
            }
            f(
                &self.app_state,
                self.is_exit(),
                &self.state_change_way,
            );
            self.state_change_way = StateChangeWay::None;
        }

        pub fn should_clear(&self, other: &StateMark) -> bool {
            self.hierarchy < other.1 || (self.hierarchy == other.1 && self.app_state != other.0)
        }

        pub fn replace(&mut self, to: AppState) {
            if match (self.app_state, to) {
                (AppState::MainMenu, AppState::MainMenu) | (AppState::InGame, AppState::InGame) => {
                    true
                }
                _ => false,
            } {
                unreachable_release!(
                    "Already in that major state or target state hierarchy is not 0"
                );
            } else if self.state_change_way != StateChangeWay::None {
                unreachable_release!("Already in state transition");
            }
            self.app_state = to;
            self.hierarchy.reset();
            self.state_change_way = StateChangeWay::Replace;
        }

        pub fn push<Child: PushState>(&mut self, child: Child) {
            if self.state_change_way != StateChangeWay::None {
                unreachable_release!("Already in state transition");
            }
            child.push(&mut self.app_state);
            self.hierarchy.increment();
            self.state_change_way = StateChangeWay::Push;
        }

        pub fn push_exit(&mut self) {
            if self.state_change_way != StateChangeWay::None {
                unreachable_release!("Already in state transition");
            }
            self.hierarchy.set_exit(true);
            self.state_change_way = StateChangeWay::Push;
        }

        pub fn pop(&mut self) {
            if self.state_change_way != StateChangeWay::None {
                unreachable_release!("Already in state transition");
            }
            self.app_state = match self.app_state.pop() {
                Some(a) => a,
                _ => unreachable_release!("Already in that state"),
            };
            self.hierarchy.decrement();
            self.state_change_way = StateChangeWay::Pop;
        }

        pub fn pop_exit(&mut self) {
            if self.state_change_way != StateChangeWay::None {
                unreachable_release!("Already in state transition");
            }
            self.hierarchy.set_exit(false);
            self.state_change_way = StateChangeWay::Pop;
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
use crate::ui::{exit_close_requested, exit_no_button, exit_yes_button, setup_exit};
pub use global::*;

pub struct StatesPlugin;

impl Plugin for StatesPlugin {
    fn build(&self, app: &mut App) {
        let manage_state_system_state = ManageStateSystemState::new(&mut app.world);
        let clear_state_system_state = ClearStateSystemState::new(&mut app.world);
        app.insert_resource(manage_state_system_state)
            .insert_resource(clear_state_system_state)
            .insert_resource(GlobalState::new(AppState::MainMenu))
            //First
            .add_system_to_stage(CoreStage::First, manage_state.exclusive_system().at_start())
            .add_state_to_stage(CoreStage::First, FirstStageState::MainMenu)
            //PreUpdate
            .add_state_to_stage(CoreStage::PreUpdate, PreUpdateStageState::MainMenu)
            //Update
            .add_state_to_stage(CoreStage::Update, UpdateStageState::MainMenu)
            //PostUpdate
            .add_state_to_stage(CoreStage::PostUpdate, PostUpdateStageState::MainMenu)
            //Last
            .add_state_to_stage(CoreStage::Last, LastStageState::MainMenu)
            .add_system_set_to_stage(
                CoreStage::PreUpdate,
                SystemSet::on_enter(PreUpdateStageState::AppExit).with_system(setup_exit),
            )
            .add_system_set_to_stage(
                CoreStage::Update,
                SystemSet::on_update(UpdateStageState::AppExit)
                    .with_system(exit_no_button)
                    .with_system(exit_yes_button)
                    .with_system(exit_close_requested),
            );
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
            app_state.propagate_change(|state, is_exit, change_way| {
                if is_exit {
                    match change_way {
                        StateChangeWay::Push => {
                            first.push(FirstStageState::AppExit).unwrap();
                            pre_update.push(PreUpdateStageState::AppExit).unwrap();
                            update.push(UpdateStageState::AppExit).unwrap();
                            post_update.push(PostUpdateStageState::AppExit).unwrap();
                            last.push(LastStageState::AppExit).unwrap();
                        }
                        _ => unreachable_release!("State is interrupted"),
                    }
                } else {
                    match change_way {
                        StateChangeWay::Replace => match *state {
                            AppState::MainMenu => {
                                first.replace(FirstStageState::MainMenu).unwrap();
                                pre_update.replace(PreUpdateStageState::MainMenu).unwrap();
                                update.replace(UpdateStageState::MainMenu).unwrap();
                                post_update.replace(PostUpdateStageState::MainMenu).unwrap();
                                last.replace(LastStageState::MainMenu).unwrap();
                            }
                            AppState::InGame => {
                                first.replace(FirstStageState::InGame).unwrap();
                                pre_update.replace(PreUpdateStageState::InGame).unwrap();
                                update.replace(UpdateStageState::InGame).unwrap();
                                post_update.replace(PostUpdateStageState::InGame).unwrap();
                                last.replace(LastStageState::InGame).unwrap();
                            }
                        },
                        // StateChangeWay::Push => match *state {
                        //     AppState::MainMenu(Some(m)) => {
                        //         first.push(FirstStageState::MainMenu(Some(m))).unwrap();
                        //         pre_update
                        //             .push(PreUpdateStageState::MainMenu(Some(m)))
                        //             .unwrap();
                        //         update.push(UpdateStageState::MainMenu(Some(m))).unwrap();
                        //         post_update
                        //             .push(PostUpdateStageState::MainMenu(Some(m)))
                        //             .unwrap();
                        //         last.push(LastStageState::MainMenu(Some(m))).unwrap();
                        //     }
                        //     AppState::InGame(Some(i)) => {
                        //         first.push(FirstStageState::InGame(Some(i))).unwrap();
                        //         pre_update
                        //             .push(PreUpdateStageState::InGame(Some(i)))
                        //             .unwrap();
                        //         update.push(UpdateStageState::InGame(Some(i))).unwrap();
                        //         post_update
                        //             .push(PostUpdateStageState::InGame(Some(i)))
                        //             .unwrap();
                        //         last.push(LastStageState::InGame(Some(i))).unwrap();
                        //     }
                        //     _ => unreachable_release!("State is interrupted"),
                        // },
                        StateChangeWay::Pop => {
                            first.pop().unwrap();
                            pre_update.pop().unwrap();
                            update.pop().unwrap();
                            post_update.pop().unwrap();
                            last.pop().unwrap();
                        }
                        _ => unreachable_release!("State is interrupted"),
                    };
                }
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
