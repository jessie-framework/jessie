# About

Jessie is a low level Rust UI framework designed to produce the fastest binaries possible with an easy developer experience.

Traditional UI frameworks have a render loop that is designed to be fit for every possible app and this approach makes their dependents depend on a meta renderer that has thousands of shaders and code paths regardless of the user actually needing them.

Jessie aims to take an approach of eliminating all possible unknowns, values and outcomes that need to be computated at runtime as much as possible at compile time.

## Static by default

We aim to make UI as static as possible, that is, keeping logic predictable in such a way that components can generate their own graphics API calls and embed their vertices directly in the outputted binary. Only shaders that will get used get embedded in the final binary and all shaders get uploaded at the beginning of runtime. Every vertice and texture data is included in a single buffer that is uploaded at the beginning of runtime.

## No duct taped API

We should encourage the user to produce actually efficient code. We should not force them to wrap their data in smart pointers just because our nice meta renderer prefers it that way. Actually we may as well not use them at all. Fuck smart pointers.

## Actual cross platform

We should aim for maximum cross platform support. Apps made with Jessie should be able to work in any major platform (Linux/Unix, Windows, Android, IOS) regardless of their configuration and hardware. A feature that doesn't work properly on one platform shouldn't count as a feature in another. Developers may be free to make stylistic changes to their design per device.

## Responsive

We should aim for making responsive design as easy as possible. We should figure out ways to embed vertice data directly in the outputted program while keeping the design responsive. 

## Very minimal dependencies

We should write our own minimal opinionated set of dependencies so that we do not depend on other implementations that we do not know the behavior of. Do keep in mind however that this doesn't mean that we should go for the lowest layer of abstraction, keeping low level graphics APIs like OpenGL or Vulkan is absolutely fine. However, (as we are already doing) we can explore how some abstractions such as Wayland work under the hood so that we can write our own dependencies for them (For example we evade libwayland's lifetime management with mutexes entirely, all of our Wayland IDs are known at compile time). (Actually we MAY scratch this one yo)

## Extensible

Just because we keep our core stable by making sure that it works on all platforms doesn't necessarily mean all developers should agree with us. They may prefer to use some platform dependent features which we can bring additional support to. But for a graphical application that only concerns itself with buttons we should keep the core as stable as possible. Developers should be able to write their own shaders and sets of components and not get a penalty in runtime cost.

## Batteries included

Instead of telling people how to add a dependency, why not just include it? There should be no dumb CLI tool that does 3 things, no `build.rs` script. `cargo add jessie` or else. (still true to this day)

## Free and open source

If a company wanted to make Jessie they very well could have. They should only be counted as big users. GPL 3.0. (im cute)
