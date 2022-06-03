#![deny(unsafe_code)]
//! Plugin for defining and loading [TextureAtlas] assets.
//!
//! The definitions are added as resources (see [GenericAtlasDefinitions] or
//! [TypedAtlasDefinition]), generic over some enumeration index `T`.
//!
//! The plugin then loads the needed images as assets before creating the individual
//! [TextureAtlas].
//!
//! Finally a resource [AtlasTextures<T>] is created (for each `T`) through which the
//! [TextureAtlas] handles can be retrieved by the enumeration index `T`.
//!
//! The plugin also provides an event [AtlasTexturesEvent<T>] upon completion or failure.

use bevy::{prelude::*, sprite::TextureAtlas, utils::HashMap};
use derive_more::IsVariant;
use std::marker::PhantomData;

pub use self::definitions::*;
pub use self::systems::*;

mod definitions;
mod systems;

/// Plugin for loading and creating [TextureAtlas] from a simple definition, and providing the
/// results in a [AtlasTextures<T>] resource.
///
/// See [GenericAtlasDefinitions].
pub struct AtlasTexturePlugin<T>(PhantomData<T>);

impl<T> Plugin for AtlasTexturePlugin<T>
where
    T: Send + Sync + Eq + std::str::FromStr + std::hash::Hash + strum::VariantNames + 'static,
    <T as std::str::FromStr>::Err: std::fmt::Debug,
{
    fn build(&self, app: &mut App) {
        app.add_system(process_atlas_definitions::<T>)
            .add_asset::<GenericAtlasDefinitions>()
            .add_event::<AtlasTexturesEvent<T>>();
    }
}

impl<T> Default for AtlasTexturePlugin<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

/// Resulting resource after creating all [TextureAtlas] for some enumeration index `T`.
///
/// Example:
/// ```
/// # use bevy::prelude::*;
/// # use bevy_atlas_loader::*;
/// #
/// #[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
/// #[derive(strum::EnumVariantNames, strum::EnumString)]
/// enum MyAtlasTextures {
///     Pacman,
/// }
///
/// fn setup_game(mut commands: Commands, atlases: Res<AtlasTextures<MyAtlasTextures>>) {
///     commands
///         .spawn_bundle(SpriteSheetBundle {
///             sprite: TextureAtlasSprite {
///                 index: 0,
///                 custom_size: Some(Vec2::new(32.0, 32.0)),
///                 ..Default::default()
///             },
///             texture_atlas: atlases.handle(MyAtlasTextures::Pacman),
///             ..Default::default()
///         });
/// }
#[derive(Debug)]
pub struct AtlasTextures<T>(HashMap<T, CreatedAtlas>)
where
    T: Eq + std::hash::Hash;

#[derive(Debug, Default, Clone)]
struct CreatedAtlas {
    handle: Handle<TextureAtlas>,
    len: usize,
}

/// Event sent whenever the plugin has (re)created the defined [AtlasTextures<T>] for some `T`
/// (or failed in doing so!).
#[derive(Debug, Clone, Copy)]
pub struct AtlasTexturesEvent<T>(ResourceStatus, PhantomData<T>);

impl<T> AtlasTexturesEvent<T> {
    pub fn status(&self) -> ResourceStatus {
        self.0
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, IsVariant)]
pub enum ResourceStatus {
    Created,
    Failed,
}

impl<T> AtlasTextures<T>
where
    T: Eq + std::hash::Hash,
{
    /// Returns a cloned [TextureAtlas] handle for a specific `T`.
    pub fn handle<B: std::borrow::Borrow<T>>(&self, index: B) -> Handle<TextureAtlas> {
        self.0[index.borrow()].handle.clone_weak()
    }

    /// Returns the total number of [TextureAtlas] index' for a specific `T`.
    ///
    /// Saves you from a lookup into `Asset<TextureAtlas>`.
    #[allow(clippy::len_without_is_empty)]
    pub fn len<B: std::borrow::Borrow<T>>(&self, index: B) -> usize {
        self.0[index.borrow()].len
    }
}

impl<T> AtlasTexturesEvent<T> {
    pub fn state(&self) -> ResourceStatus {
        self.0
    }
}
