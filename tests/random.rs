use self::common::minimal_bevy_app;
use bevy_atlas_loader::AtlasTexturePlugin;

mod common;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, strum::EnumVariantNames, strum::EnumString)]
enum MyAtlasTextures {
    Pacman,
}

#[test]
fn plugin_alone_wont_cause_panic() {
    let mut app = minimal_bevy_app();
    app.add_plugin(AtlasTexturePlugin::<MyAtlasTextures>::default());
    // spin Bevy a few times...
    (0..100).for_each(|_| app.update());
}
