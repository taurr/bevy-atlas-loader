use crate::common::minimal_bevy_app;
use bevy::{prelude::*, utils::HashMap};
use bevy_atlas_loader::{
    AtlasDefinition, AtlasTexturePlugin, AtlasTextures, AtlasTexturesEvent, GridAtlasDefinition,
    TypedAtlasDefinition,
};
use std::{
    path::Path,
    sync::{atomic::AtomicBool, Arc},
};

mod common;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, strum::EnumVariantNames, strum::EnumString)]
enum MyAtlasTextures {
    Pacman,
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

#[test]
fn undefined_entries_causes_failure() {
    let mut app = minimal_bevy_app();
    app.add_plugin(AtlasTexturePlugin::<MyAtlasTextures>::default());

    // an empty definition does not cut it!
    app.add_startup_system(move |mut cmds: Commands| {
        cmds.insert_resource(TypedAtlasDefinition::<MyAtlasTextures>::from(
            [].into_iter().collect::<HashMap<String, AtlasDefinition>>(),
        ));
    });

    // add system for capturing event
    let is_failed = Arc::new(AtomicBool::new(false));
    app.add_system({
        let is_failed = is_failed.clone();
        move |mut events: EventReader<AtlasTexturesEvent<MyAtlasTextures>>| {
            for e in events.iter() {
                if e.status().is_failed() {
                    is_failed.store(true, std::sync::atomic::Ordering::Release);
                }
            }
        }
    });

    // spin Bevy a few times...
    (0..100).for_each(|_| app.update());

    // event signalling everything OK
    assert!(is_failed.load(std::sync::atomic::Ordering::Acquire));

    // resource with the loaded TextureAtlas is NOT  available
    assert!(!app
        .world
        .contains_resource::<AtlasTextures<MyAtlasTextures>>());
}

#[ignore = "Bevy Asset Server does not see invalid paths as failures, thus we can not either!"]
#[test]
fn unloadable_paths_causes_failure() {
    let mut app = minimal_bevy_app();
    app.add_plugin(AtlasTexturePlugin::<MyAtlasTextures>::default());

    app.add_startup_system(move |mut cmds: Commands| {
        cmds.insert_resource(TypedAtlasDefinition::<MyAtlasTextures>::from(
            [(
                String::from("Pacman"),
                AtlasDefinition::from(GridAtlasDefinition {
                    texture: Path::new("invalid-path.png").into(),
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

    // add system for capturing event
    let is_failed = Arc::new(AtomicBool::new(false));
    app.add_system({
        let is_failed = is_failed.clone();
        move |mut events: EventReader<AtlasTexturesEvent<MyAtlasTextures>>| {
            for e in events.iter() {
                if e.status().is_failed() {
                    is_failed.store(true, std::sync::atomic::Ordering::Release);
                }
            }
        }
    });

    // spin Bevy a few times...
    (0..100).for_each(|_| app.update());

    // unfortunately, bevy asset server does not count non-existant paths as failures :-(
    assert!(is_failed.load(std::sync::atomic::Ordering::Acquire));

    // resource with the loaded TextureAtlas is NOT  available
    assert!(!app
        .world
        .contains_resource::<AtlasTextures<MyAtlasTextures>>());
}
