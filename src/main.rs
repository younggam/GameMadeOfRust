extern crate core;

pub(crate) mod func;
pub(crate) mod macros;
pub(crate) mod states;
pub(crate) mod ui;
pub(crate) mod consts;

use crate::states::{in_game::*, main_menu::*, *};

use bevy::{prelude::*, window::WindowSettings};

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "Game made with Rust".to_string(),
            ..default()
        })
        .insert_resource(WindowSettings {
            close_when_requested: false,
            ..default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(StatesPlugin)
        //Main Menu
        .add_plugin(MainMenuPlugin)
        //In Game
        .add_plugin(InGamePlugin)
        .run();
}
