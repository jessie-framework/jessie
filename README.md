# Jessie

Jessie is a compiled, low level Rust UI framework designed to generate as minimal overhead as possible.

## How it works

`jessie` is a compiler , a build system and a programming language at the same time. The `jessie` language is a DSL on designed to extend Rust that can get inlined into `.rs` files, and the `jessie` build system merges it all together.

### Advantages

- The compiler can reason about your entire app, allowing it to make specialized optimizations.
- The languages nature makes it easy to share packages across the broader Rust ecosystem.
- The language is minimal, it is just HTML on top of Rust

### Technical features

Jessie is designed to work anywhere, the project has a commitment to only stabilize core features if they are guaranteed to work on all major platforms.(Linux/Unix, MacOS, Windows, Android, IOS). Web platforms are not a priority but they are welcome. We want to have good backend support for OpenGL ES 2.0 as it is still one of the best platforms to ensure major cross platform support. Vulkan support is also welcome, but GLES is our priority because of how Vulkan doesn't work on old, *unupdated* platforms.

Apart from that, the language is designed to allow for various optimizations (e.g. static vertice data uploaded at the beginning of runtime). We are planning to do various optimizations at the graphics API level, at compile time.

### Planned features

- `jessie` compiler reading `rustc` HIR + AST 
- Rust subset that can compile into GLSL
- Custom windowing library backends (Wayland,X11) to ensure minimal communication overhead
- `#![no_std]` support
