use crate::consts::{FONT_SCHLUBER, UI, UI_CROSSHAIR};

use std::ops::{Deref, DerefMut};

use bevy::{prelude::*, utils::hashbrown::HashMap};

///Font handle access by str.
#[derive(Resource, Default)]
pub struct Fonts(HashMap<&'static str, Handle<Font>>);

impl Deref for Fonts {
    type Target = HashMap<&'static str, Handle<Font>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Fonts {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

///Image handle access by str. Which name should be sank to whether type or path?
#[derive(Resource, Default)]
pub struct Textures([HashMap<&'static str, Handle<Image>>; 1]);

impl Deref for Textures {
    type Target = [HashMap<&'static str, Handle<Image>>; 1];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Textures {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

///Load assets and map them to str.
pub fn assets_set_up(
    asset_server: Res<AssetServer>,
    mut fonts: ResMut<Fonts>,
    mut textures: ResMut<Textures>,
) {
    use std::path::Path;
    //fonts
    let fonts_dir = Path::new("fonts");
    fonts.insert(
        FONT_SCHLUBER,
        asset_server.load(fonts_dir.join(FONT_SCHLUBER)),
    );
    //textures
    let textures_dir = Path::new("textures");
    {
        //ui
        let ui_dir = textures_dir.join("ui");
        textures[UI].insert(UI_CROSSHAIR, asset_server.load(ui_dir.join(UI_CROSSHAIR)));
    }
}
