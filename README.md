# Jessie

Jessie is a Rust framework aiming for both an enjoyable development experience and incredible performance.

## Goals of Jessie

1. Shaders must be uploaded at the beginning of runtime
2. CJK support should be a high priority
3. No heap allocations unless absolutely necessary
4. UI code should be visually different
5. State management should feel intuitive
6. Features must not be finalized unless they work basically everywhere
7. Everything that can be done at compile time, must be done at compile time.

## FAQ

### Why?

I just wanted a library that worked easily. Instead I got messy abstractions, broken cross platform support, bad styling, bad API. I want Jessie to be theoretically the best UI framework possible, although whether that is possible is up for debate.

### Why use OpenGL instead of Vulkan or something?

Because OpenGL just works everywhere. Don't get me wrong I love Vulkan and I would implement Vulkan support if I had the time, but GLES2 just works on every device I have

### Are you overengineering?

Yes.
