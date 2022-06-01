# Bevy-atlas-loader

**WIP**, not yet released...

This crate enables the developer to define a number of [TextureAtlas] for use as sprites in [Bevy].

The atlas definition supports several styles of atlas':

1) The atlas may be specified as a grid from a texture.
2) Atlas can use random manually specified positions inside a texture.
3) An atlas can be made from a folder of textures - note this style is not supported via web.
4) (TBD) The atlas can be specified from a list of files.

If loading the atlas definition as an asset using e.g. [bevy_common_assets], the developer can 
define atlas' in a file like this:

```ron
({
    "Pacman": (
        texture: "Pac-Man.png",
        width: 19,
        height: 19,
        positions: [
            (65, 86),
            (86, 86),
            (107, 86),
        ]
    )
})
```

And utilizing Bevy's hot-reloading capability we can experiment and adjust without recompiling.

## Requirements

In order to use this crate, you need to add a few crates to you `Cargo.toml`: [bevy-atlas-loader]
and [strum].

```toml
[dependencies]
bevy-atlas-loader = "<insert version>"
strum = "<insert version>"
```

**Optional**

For defining a custom asset loader and thus loading definitions as assets, it's recommended to
use e.g. [bevy_common_assets]:
 
```toml
[dependencies]
bevy_common_assets = "<insert version>"
```

## Usage

Please have a look at the examples, and even the tests.

**TODO:** more tests

**TODO:** documentation

**TODO:** lift generic requirements?

**TODO:** github CI

**TODO:** github/crates.io links in README.md


---
This project uses [Bevy], and was bootstrapped using [bevy-template.rs].

[bevy]:https://bevyengine.org
[TextureAtlas]:https://docs.rs/bevy/latest/bevy/sprite/struct.TextureAtlas.html
[bevy-template.rs]:https://github.com/taurr/bevy-template-rs
[bevy_common_assets]: https://crates.io/crates/bevy_common_assets
[bevy-atlas-loader]:https://crates.io/crates/strum
[strum]:https://crates.io/crates/strum
[Traits]:https://doc.rust-lang.org/book/ch10-02-traits.html
