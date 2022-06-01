use std::path::Path;

use bevy::{
    asset::AssetPlugin, core_pipeline::CorePipelinePlugin, prelude::*, render::RenderPlugin,
    sprite::SpritePlugin, utils::HashMap, window::WindowPlugin,
};
use bevy_atlas_loader::{
    AtlasDefinition, AtlasTexturePlugin, AtlasTextures, GenericAtlasDefinitions,
    GridAtlasDefinition, TypedAtlasDefinition,
};
use bevy_common_assets::ron::RonAssetPlugin;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, strum::EnumVariantNames, strum::EnumString)]
enum MyAtlasTextures {
    Pacman,
}

#[test]
fn plugin_alone_wont_cause_panic() {
    let mut app = minimal_bevy_app();
    app.add_plugin(AtlasTexturePlugin::<MyAtlasTextures>::default());
    // spin Bevy a few times...
    (0..100).for_each(|_| app.update());
}

#[test]
fn definition_can_be_loaded_as_asset() {
    let mut app = minimal_bevy_app();
    app.add_plugin(AtlasTexturePlugin::<MyAtlasTextures>::default());

    // add needed 3'rd party plugin for loading definition as asset
    app.add_plugin(RonAssetPlugin::<GenericAtlasDefinitions>::new(&[
        "atlasmap",
    ]));

    // add system for adding our atlas definitions as a loaded asset
    app.add_startup_system(move |mut cmds: Commands, assets: Res<AssetServer>| {
        cmds.insert_resource(TypedAtlasDefinition::<MyAtlasTextures>::from(
            assets.load("sprite_sheets.atlasmap"),
        ));
    });

    // spin Bevy a few times...
    (0..100).for_each(|_| app.update());

    // and use the loaded TextureAtlas through the added resource
    let resource = app
        .world
        .get_resource::<AtlasTextures<MyAtlasTextures>>()
        .unwrap();
    let _texture_atlas_handle = &resource[MyAtlasTextures::Pacman];
}

#[test]
fn definition_can_be_specified_manually() {
    let mut app = minimal_bevy_app();
    app.add_plugin(AtlasTexturePlugin::<MyAtlasTextures>::default());

    // add system for adding our atlas definition
    app.add_startup_system(move |mut cmds: Commands| {
        cmds.insert_resource(TypedAtlasDefinition::<MyAtlasTextures>::from(
            [(
                String::from("Pacman"),
                AtlasDefinition::from(GridAtlasDefinition {
                    texture: Path::new("Pac-Man.png").into(),
                    columns: 3,
                    rows: 3,
                    tile_size: (19, 19),
                    padding: None,
                    ..Default::default()
                }),
            )]
            .into_iter()
            .collect::<HashMap<String, AtlasDefinition>>(),
        ));
    });

    // spin Bevy a few times...
    (0..100).for_each(|_| app.update());

    // and use the loaded TextureAtlas through the added resource
    let resource = app
        .world
        .get_resource::<AtlasTextures<MyAtlasTextures>>()
        .unwrap();
    let _texture_atlas_handle = &resource[MyAtlasTextures::Pacman];
}

#[ignore]
#[test]
fn failure_creating_atlas_can_be_detected() {
    let mut app = minimal_bevy_app();
    app.add_plugin(AtlasTexturePlugin::<MyAtlasTextures>::default());
    todo!();
}

#[ignore]
#[test]
fn failure_loading_atlas_can_be_detected() {
    let mut app = minimal_bevy_app();
    app.add_plugin(AtlasTexturePlugin::<MyAtlasTextures>::default());
    todo!();
}

fn minimal_bevy_app() -> App {
    let mut app = App::default();
    app.add_plugins(MinimalPlugins)
        .add_plugin(WindowPlugin::default())
        .add_plugin(AssetPlugin::default())
        .add_plugin(RenderPlugin::default())
        .add_plugin(CorePipelinePlugin::default())
        .add_plugin(SpritePlugin::default());
    app
}
