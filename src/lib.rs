#![deny(unsafe_code)]

use bevy::{prelude::*, sprite::TextureAtlas, utils::HashMap};
use std::marker::PhantomData;

pub use self::definitions::*;
pub use self::systems::*;

mod definitions;
mod systems;

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

#[derive(Debug)]
pub struct AtlasTextures<T>(HashMap<T, Handle<TextureAtlas>>)
where
    T: Eq + std::hash::Hash;

#[derive(Debug, Clone, Copy)]
pub struct AtlasTexturesEvent<T>(ResourceStatus, PhantomData<T>);

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum ResourceStatus {
    Created,
    Failed,
}

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
    T: Copy + Eq + std::hash::Hash,
{
    type Output = Handle<TextureAtlas>;

    fn index(&self, index: T) -> &Self::Output {
        self.0.get(&index).unwrap()
    }
}

impl<T> AtlasTexturesEvent<T> {
    pub fn state(&self) -> ResourceStatus {
        self.0
    }
}
