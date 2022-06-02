use bevy::{
    asset::AssetPlugin, core_pipeline::CorePipelinePlugin, prelude::*, render::RenderPlugin,
    sprite::SpritePlugin, window::WindowPlugin, MinimalPlugins,
};

pub(crate) fn minimal_bevy_app() -> App {
    let mut app = App::default();
    app.add_plugins(MinimalPlugins)
        .add_plugin(WindowPlugin::default())
        .add_plugin(AssetPlugin::default())
        .add_plugin(RenderPlugin::default())
        .add_plugin(CorePipelinePlugin::default())
        .add_plugin(SpritePlugin::default());
    app
}
