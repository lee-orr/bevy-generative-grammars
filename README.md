# About
This is a library for handling generative grammars in bevy.

At the moment, I'm starting with simple grammars - the first one is based on [Tracery](https://github.com/galaxykate/tracery), which is primarily useful for strings. 

However, I intend to add some other structures, to open the door for more complex options such as geometry grammars and graph grammars.

For documentation - you can find [the Book](https://lee-orr.github.io/bevy-generative-grammars) and [the API Docs](https://lee-orr.github.io/bevy-generative-grammars/doc/bevy_generative_grammars/index.html)

# Installation
To install, you currently need to specify the github repo in `Cargo.toml`:
```toml
[dependencies]
bevy = "0.9"
bevy-generative-grammars = { git = "https://github.com/lee-orr/bevy-generative-grammars", features = ["bevy"]}
```