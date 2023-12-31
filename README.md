# OSC
## Open Sound Control

## Example

An example from the [official OSC spec](https://opensoundcontrol.stanford.edu/spec-1_0-examples.html#osc-message-examples) that's [tested in CI](https://github.com/wrsturgeon/osc/blob/main/src/test.rs#L71):
```rust
let osc = (1000, -1, "hello", 1.234, 5.678).into_osc([], "foo")?;
let by_hand = b"\
    /foo\0\0\0\0\
    ,iisff\0\0\
    \x00\x00\x03\xE8\
    \xFF\xFF\xFF\xFF\
    hello\0\0\0\
    \x3F\x9D\xF3\xB6\
    \x40\xB5\xB2\x2D";

// The bundle is an iterator that never allocates a temporary array.
// We can use `Iterator::eq` to compare it to the intended output:
assert!(osc.into_iter().eq(by_hand.iter().copied()));

// Et voila !
```

## `no_std`

This library is fully `no_std` and remains fully operational without heap allocation.

Note that disabling the `alloc` feature (enabled by default) will not change any existing code,
but it will prevent you from working with OSC data whose types you don't know a priori.

If you're planning on processing unforeseen messages (not tossing them), you should keep `alloc`.

## Why another OSC library?

Practice, and I wanted a library that's easy for me to understand with a different API.

For another option, please see [rosc](https://github.com/klingtnet/rosc)!

## Property testing?

Beat me to it! Just enable the `quickcheck` feature and all types in the crate are arbitrary and shrinkable.
