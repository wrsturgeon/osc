/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Integer, float, string, or blob.

use crate::{Batch, Batched, IntoOsc};
use core::iter::{Chain, Copied};

//////////////// Trait definition

/// Integer, float, string, or blob.
pub trait Atomic:
    From<Self::AsRust> + Into<Self::AsRust> + IntoIterator<Item = u8, IntoIter = Batched<Self::Iter>>
{
    /// OSC type tag: a single character denoting this type.
    const TYPE_TAG: u8;
    /// Rust representation of this OSC type (e.g. `Integer` -> `i32`).
    type AsRust: IntoAtomic<AsAtomic = Self, AsOsc = (Self,)>;
    /// Convert from OSC to a value Rust can work with.
    #[inline(always)]
    fn into_rust(self) -> Self::AsRust {
        self.into()
    }
    /// Convert from Rust to a value OSC can work with.
    #[inline(always)]
    fn from_rust(value: Self::AsRust) -> Self {
        value.into()
    }
    /// Iterator over the OSC-formatted value.
    type Iter: Iterator<Item = u8>;
}

//////////////// Struct definitions

/// 32-bit big-endian signed two's-complement integer.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Integer([u8; 4]);
/// 32-bit big-endian IEEE 754 floating-point number.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Float([u8; 4]);
/// Null-terminated (not your responsibility!) byte string.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct String<'s>(&'s str);
/// Arbitrary known-length collection of bytes.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Blob<'b>(&'b [u8]);

//////////////// Trait implementations

impl Atomic for Integer {
    const TYPE_TAG: u8 = b'i';
    type AsRust = i32;
    type Iter = core::array::IntoIter<u8, 4>;
}
impl Atomic for Float {
    const TYPE_TAG: u8 = b'f';
    type AsRust = f32;
    type Iter = core::array::IntoIter<u8, 4>;
}
impl<'s> Atomic for String<'s> {
    const TYPE_TAG: u8 = b's';
    type AsRust = &'s str;
    type Iter = Chain<core::str::Bytes<'s>, core::iter::Once<u8>>;
}
impl<'b> Atomic for Blob<'b> {
    const TYPE_TAG: u8 = b'b';
    type AsRust = &'b [u8];
    type Iter = Copied<core::slice::Iter<'b, u8>>;
}

//////////////// `To`/`From` implementations

impl From<i32> for Integer {
    #[inline(always)]
    fn from(value: i32) -> Self {
        Self(value.to_be_bytes())
    }
}
impl From<Integer> for i32 {
    #[inline(always)]
    fn from(value: Integer) -> Self {
        i32::from_be_bytes(value.0)
    }
}

impl From<f32> for Float {
    #[inline(always)]
    fn from(value: f32) -> Self {
        Self(value.to_be_bytes())
    }
}
impl From<Float> for f32 {
    #[inline(always)]
    fn from(value: Float) -> Self {
        f32::from_be_bytes(value.0)
    }
}

impl<'s> From<&'s str> for String<'s> {
    #[inline(always)]
    fn from(value: &'s str) -> Self {
        Self(value)
    }
}
impl<'s> From<String<'s>> for &'s str {
    #[inline(always)]
    fn from(value: String<'s>) -> Self {
        value.0
    }
}

impl<'b> From<&'b [u8]> for Blob<'b> {
    #[inline(always)]
    fn from(value: &'b [u8]) -> Self {
        Self(value)
    }
}
impl<'b> From<Blob<'b>> for &'b [u8] {
    #[inline(always)]
    fn from(value: Blob<'b>) -> Self {
        value.0
    }
}

//////////////// `IntoIterator` implementations

impl IntoIterator for Integer {
    type IntoIter = Batched<core::array::IntoIter<u8, 4>>;
    type Item = u8;
    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.0.batch()
    }
}

impl IntoIterator for Float {
    type IntoIter = Batched<core::array::IntoIter<u8, 4>>;
    type Item = u8;
    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.0.batch()
    }
}

impl<'s> IntoIterator for String<'s> {
    type IntoIter = Batched<Chain<core::str::Bytes<'s>, core::iter::Once<u8>>>;
    type Item = u8;
    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.0.bytes().chain(core::iter::once(0)).batch()
    }
}

impl<'b> IntoIterator for Blob<'b> {
    type IntoIter = Batched<Copied<core::slice::Iter<'b, u8>>>;
    type Item = u8;
    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter().copied().batch()
    }
}

//////////////// Types that one-to-one map to atomic OSC types

/// Whitelists.
mod sealed {
    /// Whitelist. Otherwise useless.
    pub trait IntoAtomic {}
    impl IntoAtomic for i32 {}
    impl IntoAtomic for f32 {}
    impl IntoAtomic for &str {}
    impl IntoAtomic for &[u8] {}
}

/// Rust types that map 1-to-1 to atomic OSC types.
#[allow(clippy::module_name_repetitions)]
pub trait IntoAtomic: sealed::IntoAtomic + IntoOsc + Sized {
    /// The OSC type that directly corresponds to this Rust type.
    type AsAtomic: Atomic<AsRust = Self>;
    /// Convert directly into the OSC representation of this Rust type.
    #[inline(always)]
    fn into_atomic(self) -> Self::AsAtomic {
        Self::AsAtomic::from_rust(self)
    }
}

impl IntoAtomic for i32 {
    type AsAtomic = Integer;
}

impl IntoAtomic for f32 {
    type AsAtomic = Float;
}

impl<'s> IntoAtomic for &'s str {
    type AsAtomic = String<'s>;
}

impl<'b> IntoAtomic for &'b [u8] {
    type AsAtomic = Blob<'b>;
}
