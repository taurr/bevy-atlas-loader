use bevy::prelude::*;
use bevy_atlas_loader::{
    atlas_textures_created, AtlasTexturePlugin, AtlasTextures, AtlasTexturesEvent,
    GenericAtlasDefinitions, ResourceStatus, TypedAtlasDefinition,
};
use bevy_common_assets::ron::RonAssetPlugin;
use iyes_loopless::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum GameState {
    Initialize,
    Running,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, strum::EnumVariantNames, strum::EnumString)]
enum AtlasTextureIndex {
    Pacman,
}

#[derive(Debug, Component, Clone, Copy)]
struct UsesAtlasTexture<T>(T);

#[derive(Debug, Component, Deref, DerefMut)]
struct AtlasAnimationTimer(Timer);

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
    .add_plugin(AtlasTexturePlugin::<AtlasTextureIndex>::default());

    app.add_loopless_state(GameState::Initialize);

    app.add_enter_system(GameState::Initialize, setup_resources)
        .add_system_set(
            ConditionSet::new()
                .run_in_state(GameState::Initialize)
                .with_system(
                    change_state(GameState::Running)
                        .run_if(atlas_textures_created::<AtlasTextureIndex>),
                )
                .into(),
        );

    app.add_enter_system(GameState::Running, setup_game)
        .add_system_set(
            ConditionSet::new()
                .run_in_state(GameState::Running)
                .with_system(animate_textures)
                .with_system(update_reloaded_textures::<AtlasTextureIndex>)
                .into(),
        );

    app.run();
}

fn change_state<T>(next_state: T) -> impl Fn(Commands)
where
    T: Copy + Send + Sync + 'static,
{
    move |mut commands: Commands| {
        commands.insert_resource(NextState(next_state));
    }
}

fn setup_resources(mut commands: Commands, asset_server: Res<AssetServer>) {
    asset_server.watch_for_changes().unwrap();
    commands.insert_resource(TypedAtlasDefinition::<AtlasTextureIndex>::from(
        asset_server.load("sprite_sheets.atlasmap"),
    ));
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
            texture_atlas: atlas_textures.handle(AtlasTextureIndex::Pacman),
            ..Default::default()
        })
        .insert(UsesAtlasTexture(AtlasTextureIndex::Pacman))
        .insert(AtlasAnimationTimer(Timer::from_seconds(1.0 / 5.0, true)));
}

fn update_reloaded_textures<T: Send + Sync + Eq + core::hash::Hash + 'static>(
    mut asset_events: EventReader<AtlasTexturesEvent<T>>,
    atlas_texture_index: Query<(Entity, &UsesAtlasTexture<T>)>,
    mut commands: Commands,
    atlas_textures: Res<AtlasTextures<T>>,
) {
    for _ in asset_events
        .iter()
        .filter(|ev| ev.state() == ResourceStatus::Created)
    {
        if let Ok((entity, index)) = atlas_texture_index.get_single() {
            commands
                .entity(entity)
                .insert(atlas_textures.handle(&index.0));
        }
    }
}

fn animate_textures(
    mut query: Query<(
        &mut AtlasAnimationTimer,
        &mut TextureAtlasSprite,
        &Handle<TextureAtlas>,
    )>,
    texture_assets: Res<Assets<TextureAtlas>>,
    time: Res<Time>,
) {
    for (mut timer, mut sprite_texture, texture_handle) in query.iter_mut() {
        if timer.tick(time.delta()).just_finished() {
            if let Some(texture_atlas) = texture_assets.get(texture_handle.id) {
                sprite_texture.index = (sprite_texture.index + 1) % texture_atlas.len();
            }
        }
    }
}
