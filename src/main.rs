pub(crate) mod consts;
pub(crate) mod func;
pub(crate) mod macros;
pub(crate) mod states;
pub(crate) mod ui;

use crate::consts::FONT_0;
use crate::states::{in_game::*, main_menu::*, *};

use bevy::utils::hashbrown::HashMap;
use bevy::{prelude::*, window::WindowSettings};

use bevy_polyline::PolylinePlugin;

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "Game made with Rust".to_owned(),
            ..default()
        })
        .insert_resource(WindowSettings {
            close_when_requested: false,
            ..default()
        })
        .add_plugins(DefaultPlugins)
        .init_resource::<Fonts>()
        .add_startup_system(start_up)
        .add_plugin(PolylinePlugin)
        .add_plugin(StatesPlugin)
        //Main Menu
        .add_plugin(MainMenuPlugin)
        //In Game
        .add_plugin(InGamePlugin)
        .run();
}

pub(crate) type Fonts = HashMap<&'static str, Handle<Font>>;

fn start_up(asset_server: Res<AssetServer>, mut fonts: ResMut<Fonts>) {
    use std::path::Path;
    //fonts
    let fonts_dir = Path::new("fonts");
    fonts.insert(FONT_0, asset_server.load(fonts_dir.join(FONT_0)));
}
