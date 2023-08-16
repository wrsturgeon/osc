/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Integer, float, string, or blob.

use crate::{Batch, Batched, Dynamic, IntoOsc};
use core::iter::{once, Chain, Copied, Once};

//////////////// Trait definition

/// Integer, float, string, or blob.
pub trait Atomic:
    From<Self::AsRust> + Into<Self::AsRust> + IntoIterator<Item = u8, IntoIter = Batched<Self::Iter>>
{
    /// OSC type tag: a single character denoting this type.
    fn type_tag(&self) -> u8;
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

/// Null-terminated (not your responsibility!) byte string.
#[cfg(any(test, feature = "alloc"))]
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DynamicString(alloc::string::String);
/// Arbitrary known-length collection of bytes.
#[allow(unused_qualifications)]
#[cfg(any(test, feature = "alloc"))]
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DynamicBlob(alloc::vec::Vec<u8>);

//////////////// Trait implementations

impl Atomic for Integer {
    #[inline(always)]
    fn type_tag(&self) -> u8 {
        b'i'
    }
    type AsRust = i32;
    type Iter = core::array::IntoIter<u8, 4>;
}
impl Atomic for Float {
    #[inline(always)]
    fn type_tag(&self) -> u8 {
        b'f'
    }
    type AsRust = f32;
    type Iter = core::array::IntoIter<u8, 4>;
}
impl<'s> Atomic for String<'s> {
    #[inline(always)]
    fn type_tag(&self) -> u8 {
        b's'
    }
    type AsRust = &'s str;
    type Iter = Chain<core::str::Bytes<'s>, Once<u8>>;
}
impl<'b> Atomic for Blob<'b> {
    #[inline(always)]
    fn type_tag(&self) -> u8 {
        b'b'
    }
    type AsRust = &'b [u8];
    type Iter = Copied<core::slice::Iter<'b, u8>>;
}

#[cfg(any(test, feature = "alloc"))]
impl Atomic for Dynamic {
    #[inline(always)]
    fn type_tag(&self) -> u8 {
        match self {
            &Dynamic::Integer(ref i) => i.type_tag(),
            &Dynamic::Float(ref f) => f.type_tag(),
            &Dynamic::String(ref s) => s.type_tag(),
            &Dynamic::Blob(ref b) => b.type_tag(),
        }
    }
    type AsRust = Dynamic;
    type Iter = alloc::vec::IntoIter<u8>;
}
#[cfg(any(test, feature = "alloc"))]
impl Atomic for DynamicString {
    #[inline(always)]
    fn type_tag(&self) -> u8 {
        b's'
    }
    type AsRust = alloc::string::String;
    type Iter = Chain<alloc::vec::IntoIter<u8>, Once<u8>>;
}
#[cfg(any(test, feature = "alloc"))]
impl Atomic for DynamicBlob {
    #[inline(always)]
    fn type_tag(&self) -> u8 {
        b'b'
    }
    #[allow(unused_qualifications)]
    type AsRust = alloc::vec::Vec<u8>;
    type Iter = alloc::vec::IntoIter<u8>;
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
impl<'b, const N: usize> From<&'b [u8; N]> for Blob<'b> {
    #[inline(always)]
    fn from(value: &'b [u8; N]) -> Self {
        Self(value)
    }
}
impl<'b> From<Blob<'b>> for &'b [u8] {
    #[inline(always)]
    fn from(value: Blob<'b>) -> Self {
        value.0
    }
}

#[cfg(any(test, feature = "alloc"))]
impl From<alloc::string::String> for DynamicString {
    #[inline(always)]
    fn from(value: alloc::string::String) -> Self {
        Self(value)
    }
}
#[cfg(any(test, feature = "alloc"))]
impl From<DynamicString> for alloc::string::String {
    #[inline(always)]
    fn from(value: DynamicString) -> Self {
        value.0
    }
}

#[cfg(any(test, feature = "alloc"))]
#[allow(unused_qualifications)]
impl From<alloc::vec::Vec<u8>> for DynamicBlob {
    #[inline(always)]
    fn from(value: alloc::vec::Vec<u8>) -> Self {
        Self(value)
    }
}
#[cfg(any(test, feature = "alloc"))]
#[allow(unused_qualifications)]
impl From<DynamicBlob> for alloc::vec::Vec<u8> {
    #[inline(always)]
    fn from(value: DynamicBlob) -> Self {
        value.0
    }
}

//////////////// `IntoIterator` implementations

impl IntoIterator for Integer {
    type IntoIter = Batched<<Self as Atomic>::Iter>;
    type Item = u8;
    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.0.batch()
    }
}

impl IntoIterator for Float {
    type IntoIter = Batched<<Self as Atomic>::Iter>;
    type Item = u8;
    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.0.batch()
    }
}

impl IntoIterator for String<'_> {
    type IntoIter = Batched<<Self as Atomic>::Iter>;
    type Item = u8;
    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.0.bytes().chain(once(0)).batch()
    }
}

impl IntoIterator for Blob<'_> {
    type IntoIter = Batched<<Self as Atomic>::Iter>;
    type Item = u8;
    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter().copied().batch()
    }
}

#[cfg(any(test, feature = "alloc"))]
impl IntoIterator for Dynamic {
    type IntoIter = Batched<<Self as Atomic>::Iter>;
    type Item = u8;
    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        #[allow(unused_qualifications)]
        let v: alloc::vec::Vec<_> = match self {
            Dynamic::Integer(i) => i.into_iter().collect(),
            Dynamic::Float(f) => f.into_iter().collect(),
            Dynamic::String(s) => s.into_iter().collect(),
            Dynamic::Blob(b) => b.into_iter().collect(),
        };
        v.into_iter().batch()
    }
}

#[cfg(any(test, feature = "alloc"))]
impl IntoIterator for DynamicString {
    type IntoIter = Batched<<Self as Atomic>::Iter>;
    type Item = u8;
    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_bytes().into_iter().chain(once(0)).batch()
    }
}

#[cfg(any(test, feature = "alloc"))]
impl IntoIterator for DynamicBlob {
    type IntoIter = Batched<<Self as Atomic>::Iter>;
    type Item = u8;
    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter().batch()
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

    #[cfg(any(test, feature = "alloc"))]
    impl IntoAtomic for crate::Dynamic {}
    #[allow(unused_qualifications)]
    #[cfg(any(test, feature = "alloc"))]
    impl IntoAtomic for alloc::string::String {}
    #[allow(unused_qualifications)]
    #[cfg(any(test, feature = "alloc"))]
    impl IntoAtomic for alloc::vec::Vec<u8> {}
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

#[cfg(any(test, feature = "alloc"))]
impl IntoAtomic for Dynamic {
    type AsAtomic = Dynamic;
}

#[cfg(any(test, feature = "alloc"))]
impl IntoAtomic for alloc::string::String {
    type AsAtomic = DynamicString;
}

#[cfg(any(test, feature = "alloc"))]
#[allow(unused_qualifications)]
impl IntoAtomic for alloc::vec::Vec<u8> {
    type AsAtomic = DynamicBlob;
}

//////////////// QuickCheck implementations

#[cfg(any(test, feature = "quickcheck"))]
mod qc {
    use super::*;

    impl quickcheck::Arbitrary for Integer {
        #[inline]
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            i32::arbitrary(g).into_atomic()
        }
        #[inline]
        fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
            Box::new(self.into_rust().shrink().map(IntoAtomic::into_atomic))
        }
    }

    impl quickcheck::Arbitrary for Float {
        #[inline]
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            f32::arbitrary(g).into_atomic()
        }
        #[inline]
        fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
            Box::new(self.into_rust().shrink().map(IntoAtomic::into_atomic))
        }
    }

    impl quickcheck::Arbitrary for DynamicString {
        #[inline]
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            alloc::string::String::arbitrary(g).into_atomic()
        }
        #[inline]
        fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
            Box::new(
                self.clone()
                    .into_rust()
                    .shrink()
                    .map(IntoAtomic::into_atomic),
            )
        }
    }

    impl quickcheck::Arbitrary for DynamicBlob {
        #[inline]
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            alloc::vec::Vec::arbitrary(g).into_atomic()
        }
        #[inline]
        fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
            Box::new(
                self.clone()
                    .into_rust()
                    .shrink()
                    .map(IntoAtomic::into_atomic),
            )
        }
    }
}
