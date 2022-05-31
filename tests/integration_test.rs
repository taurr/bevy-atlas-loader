use bevy::{
    asset::AssetPlugin, core_pipeline::CorePipelinePlugin, prelude::*, render::RenderPlugin,
    sprite::SpritePlugin, window::WindowPlugin,
};
use bevy_atlas_loader::{
    AtlasDefinitions, AtlasTexturePlugin, AtlasTextures, GenericAtlasDefinitions,
};
use bevy_common_assets::ron::RonAssetPlugin;

#[derive(Debug, PartialEq, Eq, Hash, strum::EnumVariantNames, strum::EnumString)]
enum MyAtlasTextures {
    Pacman,
}

#[test]
fn plugin_alone_wont_cause_panic() {
    let mut app = minimal_bevy_app();
    app.add_plugin(AtlasTexturePlugin::<MyAtlasTextures>::default());
    for _ in 0..100 {
        app.update();
    }
}

#[test]
fn can_load_definitions_as_an_asset() -> anyhow::Result<()> {
    let mut app = minimal_bevy_app();
    app.add_plugin(AtlasTexturePlugin::<MyAtlasTextures>::default());

    // add needed 3'rd party plugin for loading definition as asset
    app.add_plugin(RonAssetPlugin::<GenericAtlasDefinitions>::new(&[
        "atlasmap",
    ]));

    // add system for adding our atlas definitions as a loaded asset
    app.add_startup_system(move |mut cmds: Commands, assets: Res<AssetServer>| {
        cmds.insert_resource(AtlasDefinitions::<MyAtlasTextures>::from(
            assets.load("sprite_sheets.atlasmap"),
        ));
    });

    // spin Bevy a few times...
    for _ in 0..100 {
        app.update();
    }

    // and use the loaded TextureAtlas
    let resource = app
        .world
        .get_resource::<AtlasTextures<MyAtlasTextures>>()
        .unwrap();
    let _texture_atlas_handle = &resource[MyAtlasTextures::Pacman];

    Ok(())
}

fn minimal_bevy_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugin(WindowPlugin::default())
        .add_plugin(AssetPlugin::default())
        .add_plugin(RenderPlugin::default())
        .add_plugin(CorePipelinePlugin::default())
        .add_plugin(SpritePlugin::default());
    app
}
