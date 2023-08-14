# OSC
### Open Sound Control

## Example

An example from the [official OSC spec](https://opensoundcontrol.stanford.edu/spec-1_0-examples.html#osc-message-examples) that's tested in CI:
```rust
let msg = (1000, -1, "hello", 1.234, 5.678).into_osc("foo");

/// The bundle is an iterator that never copies into a temporary array.
/// We can use `Iterator::eq` to compare it to the intended output:
assert!(msg.eq(b"\
    /foo\0\0\0\0\
    ,iisff\0\0\
    \x00\x00\x03\xE8\
    \xFF\xFF\xFF\xFF\
    hello\0\0\0\
    \x3F\x9D\xF3\xB6\
    \x40\xB5\xB2\x2D"
    .iter()
    .copied()))

/// Et voila ! Everything just works.
```

## `no_std`

This library is fully `no_std` and doesn't require `alloc`: just disable default features for this crate.

If you're planning on receiving OSC data whose types you can't know beforehand,
you might want to enable `alloc` to read into expansible buffers.

## Why another OSC library?

Practice, and I wanted a library that's easy for me to understand with a different API.

For another option, please see [rosc](https://github.com/klingtnet/rosc)!
