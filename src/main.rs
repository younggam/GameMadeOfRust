pub(crate) mod states;

use crate::states::{in_game::*, main_menu::*, *};

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(StatesPlugin)
        //Main Menu
        .add_plugin(MainMenuPlugin)
        //In Game
        .add_plugin(InGamePlugin)
        .run();
}
