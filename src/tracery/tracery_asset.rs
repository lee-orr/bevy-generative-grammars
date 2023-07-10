use bevy::prelude::*;

use super::TraceryGrammar;

/// The Tracery Asset
#[derive(Default)]
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
    /// Instantiates a new Tracery Asset Plugin
    pub fn new() -> Self {
        Self::default()
    }

    /// Enables JSON support - with the provided extensions
    #[cfg(feature = "json")]
    pub fn with_json(mut self, extensions: &'static [&'static str]) -> Self {
        self.json = Some(extensions);
        self
    }

    /// Enables RON support - with the provided extensions
    #[cfg(feature = "ron")]
    pub fn with_ron(mut self, extensions: &'static [&'static str]) -> Self {
        self.ron = Some(extensions);
        self
    }

    /// Enables `MessagePack` support - with the provided extensions
    #[cfg(feature = "msgpack")]
    pub fn with_msgpack(mut self, extensions: &'static [&'static str]) -> Self {
        self.msgpack = Some(extensions);
        self
    }

    /// Enables TOML support - with the provided extensions
    #[cfg(feature = "toml")]
    pub fn with_toml(mut self, extensions: &'static [&'static str]) -> Self {
        self.toml = Some(extensions);
        self
    }

    /// Enables YAML support - with the provided extensions
    #[cfg(feature = "yaml")]
    pub fn with_yaml(mut self, extensions: &'static [&'static str]) -> Self {
        self.yaml = Some(extensions);
        self
    }
}

impl Plugin for TraceryAssetPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "json")]
        if let Some(ext) = self.json {
            app.add_plugins(bevy_common_assets::json::JsonAssetPlugin::<TraceryGrammar>::new(ext));
        }
        #[cfg(feature = "ron")]
        if let Some(ext) = self.ron {
            app.add_plugins(bevy_common_assets::ron::RonAssetPlugin::<TraceryGrammar>::new(ext));
        }
        #[cfg(feature = "msgpack")]
        if let Some(ext) = self.msgpack {
            app.add_plugins(bevy_common_assets::msgpack::MsgPackAssetPlugin::<
                TraceryGrammar,
            >::new(ext));
        }
        #[cfg(feature = "toml")]
        if let Some(ext) = self.toml {
            app.add_plugins(bevy_common_assets::toml::TomlAssetPlugin::<TraceryGrammar>::new(ext));
        }
        #[cfg(feature = "yaml")]
        if let Some(ext) = self.yaml {
            app.add_plugins(bevy_common_assets::yaml::YamlAssetPlugin::<TraceryGrammar>::new(ext));
        }
    }
}
