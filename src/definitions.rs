use bevy::{prelude::*, reflect::TypeUuid, sprite::TextureAtlas, utils::HashMap};
use derive_more::{Constructor, Deref, DerefMut, From, IsVariant};
use enum_default::EnumDefault;
use serde::Deserialize;
use std::{marker::PhantomData, path::PathBuf};

/// Trait for getting the created [TextureAtlas] Handle from any definition
pub(crate) trait GetTextureAtlas {
    fn texture_atlas(&self) -> Option<&Handle<TextureAtlas>>;
}

/// Map with [AtlasDefinition]s for a creating a specific [AtlasTextures<T>](crate::AtlasTextures<T>)
/// resource.
///
/// Used when loading the definitions as assets using e.g.
/// [bevy_common_assets](https://crates.io/crates/bevy_common_assets).
///
/// # Example:
/// ```
/// # use bevy::{
/// #     prelude::*,
/// #     utils::HashMap,
/// #     asset::AssetPlugin,
/// #     core_pipeline::CorePipelinePlugin,
/// #     render::{settings::WgpuSettings, RenderPlugin},
/// #     sprite::SpritePlugin,
/// #     window::WindowPlugin,
/// #     MinimalPlugins,
/// # };
/// # use bevy_common_assets::ron::RonAssetPlugin;
/// # use bevy_atlas_loader::*;
/// # use std::{
/// #     path::Path,
/// #     sync::{atomic::AtomicBool, Arc},
/// # };
/// #
/// #[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
/// #[derive(strum::EnumVariantNames, strum::EnumString)]
/// enum MyAtlasTextures {
///     Pacman,
/// }
///
/// let mut app = App::default();
/// # app.add_plugins(MinimalPlugins)
/// #     .insert_resource(WgpuSettings {
/// #         backends: None,
/// #         ..Default::default()
/// #     })
/// #     .add_plugin(WindowPlugin::default())
/// #     .add_plugin(AssetPlugin::default())
/// #     .add_plugin(RenderPlugin::default())
/// #     .add_plugin(CorePipelinePlugin::default())
/// #     .add_plugin(SpritePlugin::default());
///
/// // we like to load definitions for all AtlasTexturePlugin<T> as assets
/// app.add_plugin(RonAssetPlugin::<GenericAtlasDefinitions>::new(&[
///     "atlasmap",
/// ]));
///
/// app.add_plugin(AtlasTexturePlugin::<MyAtlasTextures>::default());
/// app.add_startup_system(move |mut cmds: Commands, assets: Res<AssetServer>| {
///     cmds.insert_resource(TypedAtlasDefinition::<MyAtlasTextures>::from(
///         assets.load("sprite_sheets.atlasmap"),
///     ));
/// });
/// ```
#[derive(Debug, Deserialize, TypeUuid, Deref, DerefMut, Constructor, Default, From)]
#[uuid = "ef608653-e978-4a71-98e5-05c55911cfc0"]
pub struct GenericAtlasDefinitions(HashMap<String, AtlasDefinition>);

/// Defines how a [TextureAtlas] is to be created from 1 or more textures.
///
/// See [GenericAtlasDefinitions].
///
/// # Example:
/// ```rust
/// # use std::path::Path;
/// # use bevy_atlas_loader::*;
/// let _: AtlasDefinition = GridAtlasDefinition {
///     texture: Path::new("image.png").into(),
///     columns: 4,
///     rows: 3,
///     tile_size: (16, 16),
///     ..Default::default()
/// }.into();
/// ```
#[derive(Debug, Deserialize, From)]
#[serde(untagged)]
pub enum AtlasDefinition {
    Grid(GridAtlasDefinition),
    Manual(PatchAtlasDefinition),
    Folder(FolderAtlasDefinition),
}

/// Defines a [TextureAtlas] composed from a grid of an image.
///
/// # Example:
/// ```rust
/// # use std::path::Path;
/// # use bevy_atlas_loader::*;
/// let _ = GridAtlasDefinition {
///     texture: Path::new("image.png").into(),
///     columns: 4,
///     rows: 3,
///     tile_size: (16, 16),
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Default, Deserialize, Constructor)]
pub struct GridAtlasDefinition {
    pub texture: PathBuf,
    pub columns: usize,
    pub rows: usize,
    pub tile_size: (usize, usize),
    pub padding: Option<(usize, usize)>,
    #[doc(hidden)]
    #[serde(skip)]
    pub state: SingleTextureProcessState,
}

/// Defines a [TextureAtlas] composed as similar sized, mahually placed, regions inside an image.
///
/// # Example:
/// ```
/// # use std::path::Path;
/// # use bevy_atlas_loader::*;
/// let _ = PatchAtlasDefinition {
///     texture: Path::new("image.png").into(),
///     width: 16,
///     height: 16,
///     positions: vec![(0, 0)],
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Default, Deserialize)]
pub struct PatchAtlasDefinition {
    pub texture: PathBuf,
    pub width: usize,
    pub height: usize,
    pub positions: Vec<(usize, usize)>,
    #[doc(hidden)]
    #[serde(skip)]
    pub state: SingleTextureProcessState,
}

/// Defines a [TextureAtlas] as a series of images, read from a folder.
/// The sequence of the images is unknown, and may change each invocation.
///
/// # Example:
/// ```rust
/// # use std::path::Path;
/// # use bevy_atlas_loader::*;
/// let _ = FolderAtlasDefinition {
///     path: Path::new("imagefolder").into(),
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Default, Deserialize)]
pub struct FolderAtlasDefinition {
    pub path: PathBuf,
    #[doc(hidden)]
    #[serde(skip)]
    pub state: MultiTextureProcessState,
}

#[doc(hidden)]
#[derive(Debug, EnumDefault)]
pub enum SingleTextureProcessState {
    #[default]
    None,
    LoadingTexture(HandleUntyped),
    AtlasCreated(Handle<TextureAtlas>),
}

#[doc(hidden)]
#[derive(Debug, EnumDefault)]
pub enum MultiTextureProcessState {
    #[default]
    None,
    LoadingTextures(Vec<HandleUntyped>),
    AtlasCreated(Handle<TextureAtlas>),
}

/// Resource specifying how to create a specific [AtlasTextures<T>](crate::AtlasTextures<T>).
///
/// For an example of how to load the definition as an asset, see [GenericAtlasDefinitions].
/// # Example:
/// ```
/// # use bevy::{
/// #     prelude::*,
/// #     utils::HashMap,
/// #     asset::AssetPlugin,
/// #     core_pipeline::CorePipelinePlugin,
/// #     render::{settings::WgpuSettings, RenderPlugin},
/// #     sprite::SpritePlugin,
/// #     window::WindowPlugin,
/// #     MinimalPlugins,
/// # };
/// # use bevy_atlas_loader::*;
/// # use std::{
/// #     path::Path,
/// #     sync::{atomic::AtomicBool, Arc},
/// # };
/// #
/// #[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
/// #[derive(strum::EnumVariantNames, strum::EnumString)]
/// enum MyAtlasTextures {
///     Pacman,
/// }
///
/// let mut app = App::default();
/// # app.add_plugins(MinimalPlugins)
/// #     .insert_resource(WgpuSettings {
/// #         backends: None,
/// #         ..Default::default()
/// #     })
/// #     .add_plugin(WindowPlugin::default())
/// #     .add_plugin(AssetPlugin::default())
/// #     .add_plugin(RenderPlugin::default())
/// #     .add_plugin(CorePipelinePlugin::default())
/// #     .add_plugin(SpritePlugin::default());
/// app.add_plugin(AtlasTexturePlugin::<MyAtlasTextures>::default());
///
/// app.add_startup_system(move |mut cmds: Commands| {
///     cmds.insert_resource(TypedAtlasDefinition::<MyAtlasTextures>::from(
///         [(
///             String::from("Pacman"),
///             AtlasDefinition::from(GridAtlasDefinition {
///                 texture: Path::new("Pac-Man.png").into(),
///                 columns: 3,
///                 rows: 3,
///                 tile_size: (19, 19),
///                 ..Default::default()
///             }),
///         )].into_iter().collect::<HashMap<String, AtlasDefinition>>(),
///     ));
/// });
#[derive(Debug)]
#[allow(unused)]
pub struct TypedAtlasDefinition<T> {
    pub(crate) definitions: DefinitionsType,
    pub(crate) state: DefinitionProcessState,
    _marker: PhantomData<T>,
}

#[derive(Debug)]
pub(crate) enum DefinitionsType {
    Direct(Box<GenericAtlasDefinitions>),
    Indirect(Handle<GenericAtlasDefinitions>),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, IsVariant)]
pub(crate) enum DefinitionProcessState {
    Loading,
    Processing,
    Finalizing,
    Done,
    Failed,
}

impl<T> From<GenericAtlasDefinitions> for TypedAtlasDefinition<T>
where
    T: Send + Sync,
{
    fn from(definitions_map: GenericAtlasDefinitions) -> Self {
        Self {
            definitions: DefinitionsType::Direct(Box::new(definitions_map)),
            state: DefinitionProcessState::Loading,
            _marker: PhantomData::default(),
        }
    }
}

impl<T> From<HashMap<String, AtlasDefinition>> for TypedAtlasDefinition<T>
where
    T: Send + Sync,
{
    fn from(definitions_map: HashMap<String, AtlasDefinition>) -> Self {
        Self {
            definitions: DefinitionsType::Direct(Box::new(GenericAtlasDefinitions::from(
                definitions_map,
            ))),
            state: DefinitionProcessState::Loading,
            _marker: PhantomData::default(),
        }
    }
}

impl<T> From<Handle<GenericAtlasDefinitions>> for TypedAtlasDefinition<T>
where
    T: Send + Sync,
{
    fn from(handle: Handle<GenericAtlasDefinitions>) -> Self {
        Self {
            definitions: DefinitionsType::Indirect(handle),
            state: DefinitionProcessState::Loading,
            _marker: PhantomData::default(),
        }
    }
}

impl GetTextureAtlas for SingleTextureProcessState {
    fn texture_atlas(&self) -> Option<&Handle<TextureAtlas>> {
        match self {
            Self::AtlasCreated(handle) => Some(handle),
            _ => None,
        }
    }
}

impl GetTextureAtlas for MultiTextureProcessState {
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
