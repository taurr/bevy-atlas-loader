use bevy::prelude::*;
use bevy_atlas_loader::{
    atlas_textures_created, AtlasDefinitions, AtlasTexturePlugin, AtlasTextures,
    AtlasTexturesEvent, GenericAtlasDefinitions,
};
use bevy_common_assets::ron::RonAssetPlugin;
use iyes_loopless::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum GameState {
    LoadingResources,
    Running,
}

#[derive(Debug, PartialEq, Eq, Hash, strum::EnumVariantNames, strum::EnumString)]
enum AtlasTextureIndex {
    Pacman,
}

#[derive(Debug, Component, Copy, Clone)]
struct UsesAtlasTexture<T>(pub T);

fn main() {
    let mut app = App::new();
    app.insert_resource(WindowDescriptor {
        title: {
            let (name, version) = (env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
            format!("{name} - v{version}")
        },
        height: 200.0,
        width: 200.0,
        ..default()
    })
    .add_plugins(DefaultPlugins);

    app.add_plugin(RonAssetPlugin::<GenericAtlasDefinitions>::new(&[
        "atlasmap",
    ]))
    .add_plugin(AtlasTexturePlugin::<AtlasTextureIndex>::new());

    app.add_loopless_state(GameState::LoadingResources);

    app.add_enter_system(GameState::LoadingResources, setup_resources)
        .add_system_set(
            ConditionSet::new()
                .run_in_state(GameState::LoadingResources)
                .run_if(atlas_textures_created::<AtlasTextureIndex>)
                .with_system(|mut commands: Commands| {
                    commands.insert_resource(NextState(GameState::Running));
                })
                .into(),
        );

    app.add_enter_system(GameState::Running, setup_game)
        .add_system_set(
            ConditionSet::new()
                .run_in_state(GameState::Running)
                .with_system(update_atlas_textures::<AtlasTextureIndex>)
                .into(),
        );

    app.run();
}

fn setup_resources(mut commands: Commands, asset_server: Res<AssetServer>) {
    asset_server.watch_for_changes().unwrap();

    let handle: Handle<GenericAtlasDefinitions> = asset_server.load("sprite_sheets.atlasmap");
    commands.insert_resource(AtlasDefinitions::<AtlasTextureIndex>::from(handle));
}

fn setup_game(mut commands: Commands, atlas_textures: Res<AtlasTextures<AtlasTextureIndex>>) {
    let camera = OrthographicCameraBundle::new_2d();
    commands.spawn_bundle(camera);

    commands
        .spawn_bundle(SpriteSheetBundle {
            sprite: TextureAtlasSprite {
                index: 0,
                custom_size: Some(Vec2::new(128.0, 128.0)),
                ..Default::default()
            },
            texture_atlas: atlas_textures[AtlasTextureIndex::Pacman].clone(),
            ..Default::default()
        })
        .insert(UsesAtlasTexture(AtlasTextureIndex::Pacman));
}

fn update_atlas_textures<T: Send + Sync + Eq + core::hash::Hash + 'static>(
    mut asset_events: EventReader<AtlasTexturesEvent<T>>,
    atlas_texture_index: Query<(Entity, &UsesAtlasTexture<T>)>,
    mut commands: Commands,
    atlas_textures: Res<AtlasTextures<T>>,
) {
    for _ev in asset_events.iter() {
        if let Ok((entity, UsesAtlasTexture(index))) = atlas_texture_index.get_single() {
            commands
                .entity(entity)
                .insert(atlas_textures[index].clone());
        }
    }
}
