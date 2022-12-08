use std::ops::{Deref, DerefMut};

use bevy::{
    prelude::{
        shape::{Cube, Plane},
        *,
    },
    utils::hashbrown::HashMap,
};

use bevy_polyline::prelude::*;

//fonts
pub const FONT_SCHLUBER: &str = "Schluber.otf";

//images
pub const IMAGE_UI: usize = 0;
pub const CROSSHAIR: &str = "crosshair.png";

//meshes
pub const MESH_BUILT_IN: usize = 0;
pub const CUBE: &str = "cube";
pub const PLANE: &str = "plane";

//standard materials
pub const S_MAT_BUILT_IN: usize = 0;
pub const WHITE: &str = "white";
pub const WHITE_TRANS: &str = "white_trans";
pub const SEA_GREEN: &str = "sea_green";

//polylines
pub const UNIT_X: &str = "unit_x";

//polyline materials
pub const RED: &str = "red";
pub const GREEN: &str = "green";
pub const BLUE: &str = "blue";

pub struct AssetManagingPlugin;

impl Plugin for AssetManagingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Fonts>()
            .init_resource::<Images>()
            .init_resource::<Meshes>()
            .init_resource::<StandardMaterials>()
            .init_resource::<Polylines>()
            .init_resource::<PolylineMaterials>()
            .add_startup_system(assets_set_up);
    }
}

macro_rules! impl_handle_container {
    ($(#[$meta:meta])* $name:ident, $handle:ident) => {
        $(#[$meta])*
        #[derive(Resource, Default)]
        pub struct $name(HashMap<&'static str, Handle<$handle>>);

        impl Deref for $name {
            type Target = HashMap<&'static str, Handle<$handle>>;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl DerefMut for $name {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }
    };
    ($(#[$meta:meta])* $name:ident, $handle:ident, $len:literal) => {
        $(#[$meta])*
        #[derive(Resource, Default)]
        pub struct $name([HashMap<&'static str, Handle<$handle>>; $len]);

        impl Deref for $name {
            type Target = [HashMap<&'static str, Handle<$handle>>; $len];

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl DerefMut for $name {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }
    };
}

impl_handle_container!(
    ///Font handle access by str.
    Fonts,
    Font
);

impl_handle_container!(
    ///Image handle access by str. Should index name be sank to whether type or path?
    Images,
    Image,
    1
);

impl_handle_container!(
    ///Mesh handle access by str. Should index name be sank to whether type or path?
    Meshes,
    Mesh,
    1
);

impl_handle_container!(
    ///StandardMaterial handle access by str. Should index name be sank to whether type or path?
    StandardMaterials,
    StandardMaterial,
    1
);

impl_handle_container!(
    ///Polyline handle access by str. Should index name be sank to whether type or path?
    Polylines,
    Polyline
);

impl_handle_container!(
    ///PolylineMaterial handle access by str. Should index name be sank to whether type or path?
    PolylineMaterials,
    PolylineMaterial
);

///Load assets and map them to str.
#[allow(const_item_mutation)]
pub fn assets_set_up(
    asset_server: Res<AssetServer>,
    mut fonts: ResMut<Fonts>,
    mut textures: ResMut<Images>,
    mut mesh_assets: ResMut<Assets<Mesh>>,
    mut meshes: ResMut<Meshes>,
    mut standard_material_assets: ResMut<Assets<StandardMaterial>>,
    mut standard_materials: ResMut<StandardMaterials>,
    mut polyline_assets: ResMut<Assets<Polyline>>,
    mut polylines: ResMut<Polylines>,
    mut polyline_material_assets: ResMut<Assets<PolylineMaterial>>,
    mut polyline_materials: ResMut<PolylineMaterials>,
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
        textures[IMAGE_UI].insert(CROSSHAIR, asset_server.load(ui_dir.join(CROSSHAIR)));
    }
    //meshes
    {
        //builtin
        meshes[MESH_BUILT_IN].insert(CUBE, mesh_assets.add(Cube::new(1.).into()));
        meshes[MESH_BUILT_IN].insert(PLANE, mesh_assets.add(Plane { size: 1. }.into()));
    }
    //materials
    {
        //builtin
        standard_materials[S_MAT_BUILT_IN]
            .insert(WHITE, standard_material_assets.add(Color::WHITE.into()));
        standard_materials[S_MAT_BUILT_IN].insert(
            WHITE_TRANS,
            standard_material_assets.add((*Color::WHITE.set_a(0.4)).into()),
        );
        standard_materials[S_MAT_BUILT_IN].insert(
            SEA_GREEN,
            standard_material_assets.add(Color::SEA_GREEN.into()),
        );
    }
    //polylines
    polylines.insert(
        UNIT_X,
        polyline_assets.add(Polyline {
            vertices: vec![Vec3::ZERO, Vec3::X],
        }),
    );
    //polyline materials
    polyline_materials.insert(
        RED,
        polyline_material_assets.add(PolylineMaterial {
            color: Color::RED,
            perspective: true,
            ..default()
        }),
    );
    polyline_materials.insert(
        GREEN,
        polyline_material_assets.add(PolylineMaterial {
            color: Color::GREEN,
            perspective: true,
            ..default()
        }),
    );
    polyline_materials.insert(
        BLUE,
        polyline_material_assets.add(PolylineMaterial {
            color: Color::BLUE,
            perspective: true,
            ..default()
        }),
    );
}
