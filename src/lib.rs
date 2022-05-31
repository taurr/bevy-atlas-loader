use bevy::{asset::LoadState, prelude::*, sprite::TextureAtlas, utils::HashMap};
use derive_more::{Deref, From};
use serde::Deserialize;
use std::{any::type_name, marker::PhantomData, path::PathBuf};
use strum::VariantNames;

pub struct AtlasTexturePlugin<T>(PhantomData<T>);

impl<T> AtlasTexturePlugin<T> {
    pub fn new() -> Self {
        Self(Default::default())
    }
}

impl<T> Default for AtlasTexturePlugin<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Plugin for AtlasTexturePlugin<T>
where
    T: Send + Sync + Eq + std::str::FromStr + std::hash::Hash + strum::VariantNames + 'static,
    <T as std::str::FromStr>::Err: std::fmt::Debug,
{
    fn build(&self, app: &mut App) {
        app.add_system(process_atlas_definitions::<T>)
            .add_event::<AtlasTexturesEvent<T>>();
    }
}

#[derive(Debug)]
pub struct AtlasTextures<T>(HashMap<T, Handle<TextureAtlas>>)
where
    T: Eq + std::hash::Hash;

impl<T> std::ops::Index<&T> for AtlasTextures<T>
where
    T: Eq + std::hash::Hash,
{
    type Output = Handle<TextureAtlas>;

    fn index(&self, index: &T) -> &Self::Output {
        self.0.get(index).unwrap()
    }
}

impl<T> std::ops::Index<T> for AtlasTextures<T>
where
    T: Eq + std::hash::Hash,
{
    type Output = Handle<TextureAtlas>;

    fn index(&self, index: T) -> &Self::Output {
        self.0.get(&index).unwrap()
    }
}

#[derive(Debug, Deref)]
pub struct AtlasTexturesEvent<T>(#[deref] ResourceStatus, PhantomData<T>);

#[derive(Debug)]
pub enum ResourceStatus {
    Created,
    Failed,
}

/// Asset with generic map of named [AtlasDefinition]s for a specific [AtlasDefinitions<T>].
///
/// Should be loaded as an asset (by using e.g. [bevy_common_assets](https://crates.io/crates/bevy_common_assets)),
/// then inserted as a specific [AtlasDefinitions<T>] resource.
#[derive(Debug, Deserialize, bevy::reflect::TypeUuid, Deref, DerefMut)]
#[uuid = "ef608653-e978-4a71-98e5-05c55911cfc0"]
pub struct GenericAtlasDefinitions(HashMap<String, AtlasDefinition>);

impl GenericAtlasDefinitions {
    #[allow(unused)]
    pub fn new() -> Self {
        GenericAtlasDefinitions(Default::default())
    }
}

impl Default for GenericAtlasDefinitions {
    fn default() -> Self {
        Self::new()
    }
}

/// Resource for a specific [GenericAtlasDefinitions] asset handle.
///
/// Using this indirection allows to use the [AssetServer](https://docs.rs/bevy_asset/latest/bevy_asset/struct.AssetServer.html)
/// to load the definition.
#[allow(unused)]
pub struct AtlasDefinitions<T> {
    handle: Handle<GenericAtlasDefinitions>,
    state: AtlasDefinitionState,
    _marker: PhantomData<T>,
}

impl<T> From<Handle<GenericAtlasDefinitions>> for AtlasDefinitions<T>
where
    T: Send + Sync,
{
    fn from(handle: Handle<GenericAtlasDefinitions>) -> Self {
        Self {
            handle,
            state: AtlasDefinitionState::default(),
            _marker: PhantomData::default(),
        }
    }
}

/// Defines how a [TextureAtlas](https://docs.rs/bevy/latest/bevy/sprite/struct.TextureAtlas.html)
/// is to be created from 1 or more textures.
#[derive(Debug, Deserialize, From)]
#[serde(untagged)]
pub enum AtlasDefinition {
    Grid(GridAtlasDefinition),
    Manual(PatchAtlasDefinition),
    Folder(FolderAtlasDefinition),
    // TODO: FileList
}

#[derive(Debug, Deserialize, Default)]
pub struct GridAtlasDefinition {
    pub texture: PathBuf,
    pub columns: usize,
    pub rows: usize,
    pub tile_size: (usize, usize),
    pub padding: Option<(usize, usize)>,
    #[serde(skip)]
    state: SingleTextureAtlasState,
}

#[derive(Debug, Deserialize, Default)]
pub struct PatchAtlasDefinition {
    pub texture: PathBuf,
    pub width: usize,
    pub height: usize,
    pub positions: Vec<(usize, usize)>,
    #[serde(skip)]
    state: SingleTextureAtlasState,
}

#[derive(Debug, Deserialize, Default)]
pub struct FolderAtlasDefinition {
    pub path: PathBuf,
    #[serde(skip)]
    state: MultiTextureAtlasState,
}

#[derive(Debug, PartialEq)]
enum AtlasDefinitionState {
    Loading,
    Processing,
    Finalizing,
    Done,
    Failed,
}

impl Default for AtlasDefinitionState {
    fn default() -> Self {
        AtlasDefinitionState::Loading
    }
}

#[derive(Debug)]
enum SingleTextureAtlasState {
    None,
    LoadingTexture(HandleUntyped),
    AtlasCreated(Handle<TextureAtlas>),
}

impl Default for SingleTextureAtlasState {
    fn default() -> Self {
        SingleTextureAtlasState::None
    }
}

#[derive(Debug)]
enum MultiTextureAtlasState {
    None,
    LoadingTextures(Vec<HandleUntyped>),
    AtlasCreated(Handle<TextureAtlas>),
}

impl Default for MultiTextureAtlasState {
    fn default() -> Self {
        MultiTextureAtlasState::None
    }
}

trait GetTextureAtlas {
    fn texture_atlas(&self) -> Option<&Handle<TextureAtlas>>;
}

impl GetTextureAtlas for SingleTextureAtlasState {
    fn texture_atlas(&self) -> Option<&Handle<TextureAtlas>> {
        match self {
            Self::AtlasCreated(handle) => Some(handle),
            _ => None,
        }
    }
}

impl GetTextureAtlas for MultiTextureAtlasState {
    fn texture_atlas(&self) -> Option<&Handle<TextureAtlas>> {
        match self {
            Self::AtlasCreated(handle) => Some(handle),
            _ => None,
        }
    }
}

impl GetTextureAtlas for AtlasDefinition {
    fn texture_atlas(&self) -> Option<&Handle<TextureAtlas>> {
        match self {
            AtlasDefinition::Grid(d) => d.state.texture_atlas(),
            AtlasDefinition::Manual(d) => d.state.texture_atlas(),
            AtlasDefinition::Folder(d) => d.state.texture_atlas(),
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn process_atlas_definitions<T>(
    definition_handle: Option<ResMut<AtlasDefinitions<T>>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    (mut atlas_definitions, mut texture_atlases, mut texture_images): (
        ResMut<Assets<GenericAtlasDefinitions>>,
        ResMut<Assets<TextureAtlas>>,
        ResMut<Assets<Image>>,
    ),
    (mut atlas_definition_events, mut atlas_texture_event): (
        EventReader<AssetEvent<GenericAtlasDefinitions>>,
        EventWriter<AtlasTexturesEvent<T>>,
    ),
) where
    T: VariantNames + std::str::FromStr,
    T: Eq + std::hash::Hash + Send + Sync + 'static,
    <T as std::str::FromStr>::Err: std::fmt::Debug,
{
    if let Some(mut definition_handle) = definition_handle {
        match definition_handle.state {
            AtlasDefinitionState::Loading => {
                if asset_server.get_load_state(&definition_handle.handle) == LoadState::Loaded {
                    debug!(
                        enum_type = type_name::<T>(),
                        "Verifying all AtlasDefinitions are present."
                    );
                    let atlas_definitions = atlas_definitions
                        .get(&definition_handle.handle)
                        .expect("AtlasDefinitions asset should be present.");
                    definition_handle.state = T::VARIANTS
                        .iter()
                        .filter(|&&variant| !atlas_definitions.contains_key(variant))
                        .fold(AtlasDefinitionState::Processing, |_, &variant| {
                            error!(
                                enum_type = type_name::<T>(),
                                variant, "Missing atlas definition."
                            );
                            atlas_texture_event.send(AtlasTexturesEvent::<T>(
                                ResourceStatus::Failed,
                                PhantomData::default(),
                            ));
                            AtlasDefinitionState::Failed
                        });
                }
            }
            AtlasDefinitionState::Processing => {
                if let Some(atlas_definitions) =
                    atlas_definitions.get_mut(&definition_handle.handle)
                {
                    definition_handle.state = process_generic_atlas_definitions(
                        atlas_definitions,
                        &asset_server,
                        &mut texture_atlases,
                        &mut texture_images,
                    );
                    if definition_handle.state == AtlasDefinitionState::Finalizing {
                        info!(
                            enum_type = type_name::<T>(),
                            "TextureAtlas'es created for all indexes."
                        );
                        let map = atlas_definitions.iter().map(|(key, definition)| {
                            (
                                key.clone(),
                                definition.texture_atlas().cloned().expect(
                                    "Atlas not created, though all definitions are present.",
                                ),
                            )
                        });
                        commands.insert_resource(AtlasTextures::<T>(
                            map.map(|(key, handle)| {
                                let key = T::from_str(&key).unwrap();
                                (key, handle)
                            })
                            .collect(),
                        ));
                    }
                }
            }
            AtlasDefinitionState::Finalizing => {
                atlas_texture_event.send(AtlasTexturesEvent::<T>(
                    ResourceStatus::Created,
                    PhantomData::default(),
                ));
                definition_handle.state = AtlasDefinitionState::Done
            }
            AtlasDefinitionState::Done | AtlasDefinitionState::Failed => {
                for ev in atlas_definition_events.iter() {
                    match ev {
                        AssetEvent::Created { handle } | AssetEvent::Modified { handle } => {
                            if &definition_handle.handle == handle {
                                warn!(
                                    enum_type = type_name::<T>(),
                                    "AtlasDefinition has changed. Recreating atlas."
                                );
                                definition_handle.state = AtlasDefinitionState::Loading;
                            }
                        }
                        AssetEvent::Removed { handle } => {
                            if &definition_handle.handle == handle {
                                error!(
                                    enum_type = type_name::<T>(),
                                    "AtlasDefinitions should never be removed!"
                                )
                            }
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
) -> AtlasDefinitionState {
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
        AtlasDefinitionState::Finalizing
    } else {
        AtlasDefinitionState::Processing
    }
}

fn process_folder_atlas_definition(
    folder_definition: &mut FolderAtlasDefinition,
    asset_server: &AssetServer,
    texture_atlases: &mut Assets<TextureAtlas>,
    texture_images: &mut Assets<Image>,
) -> bool {
    match &folder_definition.state {
        MultiTextureAtlasState::None => {
            folder_definition.state = MultiTextureAtlasState::LoadingTextures(
                asset_server
                    .load_folder(folder_definition.path.as_path())
                    .expect("path must exist and be a folder"),
            );
            false
        }
        MultiTextureAtlasState::LoadingTextures(handles) => {
            let mut texture_atlas_builder = TextureAtlasBuilder::default();
            for handle in handles {
                let texture = texture_images.get(handle.id).unwrap();
                texture_atlas_builder.add_texture(handle.clone().typed::<Image>(), texture);
            }
            let atlas = texture_atlas_builder.finish(texture_images).unwrap();
            folder_definition.state =
                MultiTextureAtlasState::AtlasCreated(texture_atlases.add(atlas));
            false
        }
        MultiTextureAtlasState::AtlasCreated(_) => true,
    }
}

fn process_patch_atlas_definition(
    patch_definition: &mut PatchAtlasDefinition,
    asset_server: &AssetServer,
    texture_atlases: &mut Assets<TextureAtlas>,
) -> bool {
    match &patch_definition.state {
        SingleTextureAtlasState::None => {
            patch_definition.state = SingleTextureAtlasState::LoadingTexture(
                asset_server.load_untyped(patch_definition.texture.as_path()),
            );
            false
        }
        SingleTextureAtlasState::LoadingTexture(handle) => {
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
                    SingleTextureAtlasState::AtlasCreated(texture_atlases.add(atlas));
            }
            false
        }
        SingleTextureAtlasState::AtlasCreated(_) => true,
    }
}

fn process_grid_atlas_definition(
    grid_definition: &mut GridAtlasDefinition,
    asset_server: &AssetServer,
    texture_atlases: &mut Assets<TextureAtlas>,
) -> bool {
    match &grid_definition.state {
        SingleTextureAtlasState::None => {
            grid_definition.state = SingleTextureAtlasState::LoadingTexture(
                asset_server.load_untyped(grid_definition.texture.as_path()),
            );
            false
        }
        SingleTextureAtlasState::LoadingTexture(handle) => {
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
                    SingleTextureAtlasState::AtlasCreated(texture_atlases.add(atlas));
            }
            false
        }
        SingleTextureAtlasState::AtlasCreated(_) => true,
    }
}

#[allow(unused)]
pub fn atlas_textures_failed<T>(handle: Res<AtlasDefinitions<T>>) -> bool
where
    T: Send + Sync + 'static,
{
    match handle.state {
        AtlasDefinitionState::Loading => false,
        AtlasDefinitionState::Processing => false,
        AtlasDefinitionState::Finalizing => false,
        AtlasDefinitionState::Done => false,
        AtlasDefinitionState::Failed => true,
    }
}

#[allow(unused)]
pub fn atlas_textures_created<T>(handle: Res<AtlasDefinitions<T>>) -> bool
where
    T: Send + Sync + 'static,
{
    match handle.state {
        AtlasDefinitionState::Loading => false,
        AtlasDefinitionState::Processing => false,
        AtlasDefinitionState::Finalizing => false,
        AtlasDefinitionState::Done => true,
        AtlasDefinitionState::Failed => false,
    }
}

#[cfg(test)]
mod tests {

    mod config_file {
        mod allows_format {
            use crate::*;

            type Result = anyhow::Result<()>;

            #[test]
            fn patchwork() -> Result {
                let cfg_file = indoc::indoc! {r#"
                    ({
                        "patchwork": (
                            texture: "Pac-Man.png",
                            width: 19,
                            height: 19,
                            positions: [
                                (65, 86),
                                (86, 86),
                                (107, 86),
                            ]
                        ),
                    })"#};

                let config: GenericAtlasDefinitions = ron::from_str(cfg_file)?;
                dbg!(config);
                Ok(())
            }

            #[test]
            fn grid() -> Result {
                let cfg_file = indoc::indoc! {r#"
                    ({
                        "grid": (
                            texture: "Pac-Man.png",
                            columns: 8,
                            rows: 4,
                            tile_size: (20, 20),
                            padding: None,
                        ),
                    })"#};

                let config: GenericAtlasDefinitions = ron::from_str(cfg_file)?;
                dbg!(config);
                Ok(())
            }

            #[test]
            fn folder() -> Result {
                let cfg_file = indoc::indoc! {r#"
                    ({
                        "folder": (
                            path: "texture-folder",
                        ),
                    })"#};

                let config: GenericAtlasDefinitions = ron::from_str(cfg_file)?;
                dbg!(config);
                Ok(())
            }

            #[test]
            fn multiple_of_differet_types() -> Result {
                let cfg_file = indoc::indoc! {r#"
                    ({
                        "patchwork": (
                            texture: "Pac-Man.png",
                            width: 19,
                            height: 19,
                            positions: [
                                (65, 86),
                                (86, 86),
                                (107, 86),
                            ]
                        ),
                        "grid": (
                            texture: "Pac-Man.png",
                            columns: 8,
                            rows: 4,
                            tile_size: (20, 20),
                            padding: None,
                        ),
                        "folder": (
                            path: "texture-folder",
                        ),
                    })"#};

                let config: GenericAtlasDefinitions = ron::from_str(cfg_file)?;
                dbg!(config);
                Ok(())
            }
        }
    }
}
