pub(crate) mod asset;
pub(crate) mod consts;
pub(crate) mod func;
pub(crate) mod macros;
pub(crate) mod physics;
pub(crate) mod states;
pub(crate) mod ui;

use crate::{
    asset::{
        assets_set_up, {Fonts, Textures},
    },
    states::{in_game::*, main_menu::*, *},
};

use bevy::prelude::*;

use bevy_polyline::PolylinePlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                title: "Game made with Rust".to_owned(),
                ..default()
            },
            close_when_requested: false,
            ..default()
        }))
        //Asset manage helpers
        .init_resource::<Fonts>()
        .init_resource::<Textures>()
        .add_startup_system(assets_set_up)
        //Polyline lib
        .add_plugin(PolylinePlugin)
        //Global states manager
        .add_plugin(StatesPlugin)
        //Main Menu
        .add_plugin(MainMenuPlugin)
        //In Game
        .add_plugin(InGamePlugin)
        .run();
}
