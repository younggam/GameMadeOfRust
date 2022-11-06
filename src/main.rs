pub(crate) mod consts;
pub(crate) mod func;
pub(crate) mod macros;
pub(crate) mod states;
pub(crate) mod ui;

use crate::consts::{FONT_SCHLUBER, UI, UI_CROSSHAIR};
use crate::states::{in_game::*, main_menu::*, *};

use bevy::{prelude::*, utils::hashbrown::HashMap, window::WindowSettings};

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
        .init_resource::<Textures>()
        .add_startup_system(assets_set_up)
        .add_plugin(PolylinePlugin)
        .add_plugin(StatesPlugin)
        //Main Menu
        .add_plugin(MainMenuPlugin)
        //In Game
        .add_plugin(InGamePlugin)
        .run();
}

pub(crate) type Fonts = HashMap<&'static str, Handle<Font>>;
pub(crate) type Textures = [HashMap<&'static str, Handle<Image>>; 1];

fn assets_set_up(
    asset_server: Res<AssetServer>,
    mut fonts: ResMut<Fonts>,
    mut textures: ResMut<Textures>,
) {
    use std::path::Path;
    // fonts
    let fonts_dir = Path::new("fonts");
    fonts.insert(
        FONT_SCHLUBER,
        asset_server.load(fonts_dir.join(FONT_SCHLUBER)),
    );
    // textures
    let textures_dir = Path::new("textures");
    {
        //ui
        let ui_dir = textures_dir.join("ui");
        textures[UI].insert(UI_CROSSHAIR, asset_server.load(ui_dir.join(UI_CROSSHAIR)));
    }
}
