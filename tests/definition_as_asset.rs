use crate::common::minimal_bevy_app;
use bevy::prelude::*;
use bevy_atlas_loader::{
    AtlasTexturePlugin, AtlasTextures, AtlasTexturesEvent, GenericAtlasDefinitions,
    TypedAtlasDefinition,
};
use bevy_common_assets::ron::RonAssetPlugin;
use std::sync::{atomic::AtomicBool, Arc};

mod common;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, strum::EnumVariantNames, strum::EnumString)]
enum MyAtlasTextures {
    Pacman,
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

    // add system for capturing event
    let is_created = Arc::new(AtomicBool::new(false));
    app.add_system({
        let is_created = is_created.clone();
        move |mut events: EventReader<AtlasTexturesEvent<MyAtlasTextures>>| {
            for e in events.iter() {
                if e.status().is_created() {
                    is_created.store(true, std::sync::atomic::Ordering::Release);
                }
            }
        }
    });

    // spin Bevy a few times...
    (0..100).for_each(|_| app.update());

    // event signalling everything OK
    assert!(is_created.load(std::sync::atomic::Ordering::Acquire));

    // resource with the loaded TextureAtlas is now available
    assert!(app
        .world
        .contains_resource::<AtlasTextures<MyAtlasTextures>>());
}
