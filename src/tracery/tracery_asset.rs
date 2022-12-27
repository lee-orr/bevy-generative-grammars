use bevy::prelude::*;

use super::TraceryGrammar;
use bevy_common_assets::*;

/// The Tracery Asset
pub struct TraceryAssetPlugin {
    #[cfg(feature = "json")]
    json: Option<&'static [&'static str]>,
    #[cfg(feature = "ron")]
    ron: Option<&'static [&'static str]>,
    #[cfg(feature = "msgpack")]
    msgpack: Option<&'static [&'static str]>,
    #[cfg(feature = "toml")]
    toml: Option<&'static [&'static str]>,
    #[cfg(feature = "yaml")]
    yaml: Option<&'static [&'static str]>,
}

impl TraceryAssetPlugin {
    fn new() -> Self {
        Self {
            #[cfg(feature = "json")]
            json: None,
            #[cfg(feature = "ron")]
            ron: None,
            #[cfg(feature = "msgpack")]
            msgpack: None,
            #[cfg(feature = "toml")]
            toml: None,
            #[cfg(feature = "yaml")]
            yaml: None,
        }
    }

    #[cfg(feature = "json")]
    fn with_json(mut self, extensions: &'static [&'static str]) -> Self {
        self.json = Some(extensions);
        self
    }

    #[cfg(feature = "ron")]
    fn with_ron(mut self, extensions: &'static [&'static str]) -> Self {
        self.ron = Some(extensions);
        self
    }

    #[cfg(feature = "msgpack")]
    fn with_msgpack(mut self, extensions: &'static [&'static str]) -> Self {
        self.msgpack = Some(extensions);
        self
    }

    #[cfg(feature = "toml")]
    fn with_toml(mut self, extensions: &'static [&'static str]) -> Self {
        self.toml = Some(extensions);
        self
    }

    #[cfg(feature = "yaml")]
    fn with_yaml(mut self, extensions: &'static [&'static str]) -> Self {
        self.yaml = Some(extensions);
        self
    }
}

impl Plugin for TraceryAssetPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "json")]
        if let Some(ext) = self.json {
            app.add_plugin(json::JsonAssetPlugin::<TraceryGrammar>::new(ext));
        }
        #[cfg(feature = "ron")]
        if let Some(ext) = self.ron {
            app.add_plugin(ron::RonAssetPlugin::<TraceryGrammar>::new(ext));
        }
        #[cfg(feature = "msgpack")]
        if let Some(ext) = self.msgpack {
            app.add_plugin(msgpack::MsgPackAssetPlugin::<TraceryGrammar>::new(ext));
        }
        #[cfg(feature = "toml")]
        if let Some(ext) = self.toml {
            app.add_plugin(toml::TomlAssetPlugin::<TraceryGrammar>::new(ext));
        }
        #[cfg(feature = "yaml")]
        if let Some(ext) = self.yaml {
            app.add_plugin(yaml::YamlAssetPlugin::<TraceryGrammar>::new(ext));
        }
    }
}
