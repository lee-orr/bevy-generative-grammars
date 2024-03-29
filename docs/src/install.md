# Installation

To install, add the following to your dependencies in the `Cargo.toml`:

```toml
bevy-generative-grammars = { version = "0.0.2", features = ["bevy"]}
```

## Available features

- default - this only provides the basic functionality, and relies on `std::collections::HashMap` internally.
- bevy - this implements `Resource` and `Component` for grammars & stateful generators, as well as switching to `bevy::utils::HashMap`
- serde - this provides a serialization/deserialization
- asset - you don't need to use this directly, but it's used as the backbone for the various asset plugin options.
- json - provides a JSON asset plugin
- ron - provides a RON asset plugin
- msgpack - provides a MessagePack asset plugin
- toml - provides a TOML asset plugin

- yaml - provides a YAML asset plugin
