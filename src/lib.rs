#![forbid(missing_docs)]
#![forbid(unsafe_code)]
#![warn(clippy::doc_markdown)]
#![doc = include_str!("../README.md")]

use bevy::prelude::*;

mod generator;

/// A plugin
pub struct HelloWorldPlugin;

impl Plugin for HelloWorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(hello_world);
    }
}

fn hello_world() {
    println!("Hello, World!");
}
