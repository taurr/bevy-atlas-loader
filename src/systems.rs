use bevy::{asset::LoadState, prelude::*, sprite::TextureAtlas};
use std::{any::type_name, marker::PhantomData};
use strum::VariantNames;

use crate::{
    AtlasDefinition, AtlasTextures, AtlasTexturesEvent, CreatedAtlas, DefinitionProcessState,
    FolderAtlasDefinition, GenericAtlasDefinitions, GetTextureAtlas, GridAtlasDefinition,
    MultiTextureProcessState, PatchAtlasDefinition, ResourceStatus, SingleTextureProcessState,
    TypedAtlasDefinition,
};

#[allow(unused)]
pub fn atlas_textures_failed<T>(handle: Option<Res<TypedAtlasDefinition<T>>>) -> bool
where
    T: Send + Sync + 'static,
{
    if let Some(handle) = handle {
        handle.state.is_failed()
    } else {
        false
    }
}

#[allow(unused)]
pub fn atlas_textures_created<T>(handle: Option<Res<TypedAtlasDefinition<T>>>) -> bool
where
    T: Send + Sync + 'static,
{
    if let Some(handle) = handle {
        handle.state.is_done()
    } else {
        false
    }
}

#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub fn process_atlas_definitions<T>(
    definition_handle: Option<ResMut<TypedAtlasDefinition<T>>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut atlas_definitions: ResMut<Assets<GenericAtlasDefinitions>>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut texture_images: ResMut<Assets<Image>>,
    atlas_definition_events: EventReader<AssetEvent<GenericAtlasDefinitions>>,
    mut atlas_texture_event: EventWriter<AtlasTexturesEvent<T>>,
) where
    T: VariantNames + std::str::FromStr,
    T: Eq + std::hash::Hash + Send + Sync + 'static,
    <T as std::str::FromStr>::Err: std::fmt::Debug,
{
    if let Some(mut definition_handle) = definition_handle {
        match definition_handle.state {
            DefinitionProcessState::Loading => {
                if match &definition_handle.definitions {
                    crate::DefinitionsType::Direct(_) => true,
                    crate::DefinitionsType::Indirect(handle) => {
                        asset_server.get_load_state(handle) == LoadState::Loaded
                    }
                } {
                    debug!(
                        T = type_name::<T>(),
                        "Verifying all AtlasDefinitions<T> are present."
                    );
                    let atlas_definitions = match &definition_handle.definitions {
                        crate::DefinitionsType::Direct(definitions) => definitions.as_ref(),
                        crate::DefinitionsType::Indirect(handle) => atlas_definitions
                            .get(handle.id)
                            .expect("AtlasDefinitions asset should be present."),
                    };
                    definition_handle.state = T::VARIANTS
                        .iter()
                        .filter(|&&variant| !atlas_definitions.contains_key(variant))
                        .fold(DefinitionProcessState::Processing, |_, &variant| {
                            error!(
                                T = type_name::<T>(),
                                Variant = variant,
                                variant,
                                "Missing AtlasDefinition<T> for variant."
                            );
                            let event_writer = &mut atlas_texture_event;
                            event_writer.send(AtlasTexturesEvent::<T>(
                                ResourceStatus::Failed,
                                PhantomData::default(),
                            ));
                            DefinitionProcessState::Failed
                        });
                }
            }
            DefinitionProcessState::Processing => {
                let definition_handle = &mut *definition_handle;
                let atlas_definitions = match definition_handle.definitions {
                    crate::DefinitionsType::Direct(ref mut definitions) => definitions.as_mut(),
                    crate::DefinitionsType::Indirect(ref mut handle) => atlas_definitions
                        .get_mut(handle.id)
                        .expect("AtlasDefinitions asset should be present."),
                };
                definition_handle.state = process_generic_atlas_definitions(
                    atlas_definitions,
                    &asset_server,
                    &mut texture_atlases,
                    &mut texture_images,
                );
                if definition_handle.state == DefinitionProcessState::Finalizing {
                    info!(T = type_name::<T>(), "AtlasTexture<T> created for all T.");
                    let map = atlas_definitions.iter().map(|(key, definition)| {
                        (
                            key.clone(),
                            definition
                                .texture_atlas()
                                .cloned()
                                .expect("Atlas not created, though all definitions are present."),
                        )
                    });
                    commands.insert_resource(AtlasTextures::<T>(
                        map.map(|(key, handle)| {
                            let key = T::from_str(&key).unwrap();
                            let len = texture_atlases.get(&handle).unwrap().len();
                            (key, CreatedAtlas { handle, len })
                        })
                        .collect(),
                    ));
                }
            }
            DefinitionProcessState::Finalizing => {
                let mut event_writer = atlas_texture_event;
                event_writer.send(AtlasTexturesEvent::<T>(
                    ResourceStatus::Created,
                    PhantomData::default(),
                ));
                definition_handle.state = DefinitionProcessState::Done
            }
            DefinitionProcessState::Done | DefinitionProcessState::Failed => {
                let mut event_reader = atlas_definition_events;
                for ev in event_reader.iter() {
                    match ev {
                        AssetEvent::Created { handle } | AssetEvent::Modified { handle } => {
                            match &definition_handle.definitions {
                                crate::DefinitionsType::Indirect(h) if h == handle => {
                                    warn!(
                                        T = type_name::<T>(),
                                        "AtlasDefinitions<T> has changed. Recreating atlas."
                                    );
                                    definition_handle.state = DefinitionProcessState::Loading;
                                }
                                _ => {}
                            }
                        }
                        AssetEvent::Removed { .. } => {
                            error!("AtlasDefinitions should never be removed!")
                        }
                    }
                }
            }
        }
    }
}

fn process_generic_atlas_definitions(
    atlas_definitions: &mut GenericAtlasDefinitions,
    asset_server: &AssetServer,
    texture_atlases: &mut Assets<TextureAtlas>,
    texture_images: &mut Assets<Image>,
) -> DefinitionProcessState {
    if atlas_definitions.iter_mut().all(|(_key, cfg)| match cfg {
        AtlasDefinition::Grid(grid_definition) => {
            process_grid_atlas_definition(grid_definition, asset_server, texture_atlases)
        }
        AtlasDefinition::Manual(patch_definition) => {
            process_patch_atlas_definition(patch_definition, asset_server, texture_atlases)
        }
        AtlasDefinition::Folder(folder_definition) => process_folder_atlas_definition(
            folder_definition,
            asset_server,
            texture_atlases,
            texture_images,
        ),
    }) {
        DefinitionProcessState::Finalizing
    } else {
        DefinitionProcessState::Processing
    }
}

fn process_grid_atlas_definition(
    grid_definition: &mut GridAtlasDefinition,
    asset_server: &AssetServer,
    texture_atlases: &mut Assets<TextureAtlas>,
) -> bool {
    match &grid_definition.state {
        SingleTextureProcessState::None => {
            grid_definition.state = SingleTextureProcessState::LoadingTexture(
                asset_server.load_untyped(grid_definition.texture.as_path()),
            );
            false
        }
        SingleTextureProcessState::LoadingTexture(handle) => {
            let image = handle.clone().typed::<Image>();
            if asset_server.get_load_state(&image) == LoadState::Loaded {
                let atlas = TextureAtlas::from_grid_with_padding(
                    image,
                    Vec2::new(
                        grid_definition.tile_size.0 as f32,
                        grid_definition.tile_size.1 as f32,
                    ),
                    grid_definition.columns,
                    grid_definition.rows,
                    match grid_definition.padding {
                        Some((x, y)) => Vec2::new(x as f32, y as f32),
                        None => Vec2::ZERO,
                    },
                );
                grid_definition.state =
                    SingleTextureProcessState::AtlasCreated(texture_atlases.add(atlas));
            }
            false
        }
        SingleTextureProcessState::AtlasCreated(_) => true,
    }
}

fn process_patch_atlas_definition(
    patch_definition: &mut PatchAtlasDefinition,
    asset_server: &AssetServer,
    texture_atlases: &mut Assets<TextureAtlas>,
) -> bool {
    match &patch_definition.state {
        SingleTextureProcessState::None => {
            patch_definition.state = SingleTextureProcessState::LoadingTexture(
                asset_server.load_untyped(patch_definition.texture.as_path()),
            );
            false
        }
        SingleTextureProcessState::LoadingTexture(handle) => {
            let image = handle.clone().typed::<Image>();
            if asset_server.get_load_state(&image) == LoadState::Loaded {
                let mut atlas = TextureAtlas::new_empty(
                    image,
                    Vec2::new(
                        patch_definition.width as f32,
                        patch_definition.height as f32,
                    ),
                );
                for &(x, y) in patch_definition.positions.iter() {
                    atlas.add_texture(bevy::sprite::Rect {
                        min: Vec2::new(x as f32, y as f32),
                        max: Vec2::new(
                            (x + patch_definition.width) as f32,
                            (y + patch_definition.height) as f32,
                        ),
                    });
                }
                patch_definition.state =
                    SingleTextureProcessState::AtlasCreated(texture_atlases.add(atlas));
            }
            false
        }
        SingleTextureProcessState::AtlasCreated(_) => true,
    }
}

fn process_folder_atlas_definition(
    folder_definition: &mut FolderAtlasDefinition,
    asset_server: &AssetServer,
    texture_atlases: &mut Assets<TextureAtlas>,
    texture_images: &mut Assets<Image>,
) -> bool {
    match &folder_definition.state {
        MultiTextureProcessState::None => {
            folder_definition.state = MultiTextureProcessState::LoadingTextures(
                asset_server
                    .load_folder(folder_definition.path.as_path())
                    .expect("path must exist and be a folder"),
            );
            false
        }
        MultiTextureProcessState::LoadingTextures(handles) => {
            let mut texture_atlas_builder = TextureAtlasBuilder::default();
            for handle in handles {
                let texture = texture_images.get(handle.id).unwrap();
                texture_atlas_builder.add_texture(handle.clone().typed::<Image>(), texture);
            }
            let atlas = texture_atlas_builder.finish(texture_images).unwrap();
            folder_definition.state =
                MultiTextureProcessState::AtlasCreated(texture_atlases.add(atlas));
            false
        }
        MultiTextureProcessState::AtlasCreated(_) => true,
    }
}
