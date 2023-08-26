/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Integer, float, string, or blob.

use crate::{Aligned4B, Batch, Batched, Decode, IntoOsc, Misaligned4B, Tag};
use core::iter::{once, Chain, Copied, Once};

#[cfg(feature = "alloc")]
use crate::Data;

//////////////// Trait definition

/// Integer, float, string, or blob.
pub trait Atomic:
    TryFrom<Self::AsRust> + Into<Self::AsRust> + IntoIterator<Item = u8, IntoIter = Batched<Self::Iter>>
where
    InvalidContents: From<<Self as TryFrom<Self::AsRust>>::Error>,
{
    /// OSC type tag: a single character denoting this type.
    fn type_tag(&self) -> Tag;
    /// Rust representation of this OSC type (e.g. `Integer` -> `i32`).
    type AsRust: IntoAtomic<AsAtomic = Self, AsOsc = (Self,)>;
    /// Convert from OSC to a value Rust can work with.
    #[inline(always)]
    fn into_rust(self) -> Self::AsRust {
        self.into()
    }
    /// Convert from Rust to a value OSC can work with.
    /// # Errors
    /// If the data is invalid (e.g. a non-ASCII string).
    #[inline(always)]
    fn from_rust(value: Self::AsRust) -> Result<Self, Self::Error> {
        value.try_into()
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
#[cfg(feature = "alloc")]
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DynamicString(alloc::string::String);
/// Arbitrary known-length collection of bytes.
#[allow(unused_qualifications)]
#[cfg(feature = "alloc")]
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DynamicBlob(alloc::vec::Vec<u8>);

//////////////// Trait implementations

impl Atomic for Integer {
    #[inline(always)]
    fn type_tag(&self) -> Tag {
        Tag::Integer
    }
    type AsRust = i32;
    type Iter = core::array::IntoIter<u8, 4>;
}
impl Atomic for Float {
    #[inline(always)]
    fn type_tag(&self) -> Tag {
        Tag::Float
    }
    type AsRust = f32;
    type Iter = core::array::IntoIter<u8, 4>;
}
impl<'s> Atomic for String<'s> {
    #[inline(always)]
    fn type_tag(&self) -> Tag {
        Tag::String
    }
    type AsRust = &'s str;
    type Iter = Chain<core::str::Bytes<'s>, Once<u8>>;
}
impl<'b> Atomic for Blob<'b> {
    #[inline(always)]
    fn type_tag(&self) -> Tag {
        Tag::Blob
    }
    type AsRust = &'b [u8];
    type Iter = Copied<core::slice::Iter<'b, u8>>;
}

#[cfg(feature = "alloc")]
impl Atomic for Data {
    #[inline(always)]
    fn type_tag(&self) -> Tag {
        match self {
            &Data::Integer(ref i) => i.type_tag(),
            &Data::Float(ref f) => f.type_tag(),
            &Data::String(ref s) => s.type_tag(),
            &Data::Blob(ref b) => b.type_tag(),
        }
    }
    type AsRust = Data;
    type Iter = alloc::vec::IntoIter<u8>;
}
#[cfg(feature = "alloc")]
impl Atomic for DynamicString {
    #[inline(always)]
    fn type_tag(&self) -> Tag {
        Tag::String
    }
    type AsRust = alloc::string::String;
    type Iter = Chain<alloc::vec::IntoIter<u8>, Once<u8>>;
}
#[cfg(feature = "alloc")]
impl Atomic for DynamicBlob {
    #[inline(always)]
    fn type_tag(&self) -> Tag {
        Tag::Blob
    }
    #[allow(unused_qualifications)]
    type AsRust = alloc::vec::Vec<u8>;
    type Iter = alloc::vec::IntoIter<u8>;
}

//////////////// `From` implementations

/// Invalid data in conversion from Rust to OSC (e.g. a non-ASCII string).
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum InvalidContents {
    /// Non-ASCII character in a Rust string.
    NonAscii,
    /// Null byte in an otherwise normal ASCII string.
    NullInString,
}

impl From<core::convert::Infallible> for InvalidContents {
    #[inline(always)]
    fn from(_: core::convert::Infallible) -> Self {
        #[cfg(test)]
        #[allow(clippy::unreachable)]
        {
            unreachable!()
        }
        #[cfg(not(test))]
        #[allow(unsafe_code)]
        // SAFETY:
        // Input to this function can never be constructed.
        unsafe {
            core::hint::unreachable_unchecked()
        }
    }
}

impl TryFrom<i32> for Integer {
    type Error = core::convert::Infallible;
    #[inline(always)]
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        Ok(Self(value.to_be_bytes()))
    }
}
impl From<Integer> for i32 {
    #[inline(always)]
    fn from(value: Integer) -> Self {
        i32::from_be_bytes(value.0)
    }
}

impl TryFrom<f32> for Float {
    type Error = core::convert::Infallible;
    #[inline(always)]
    fn try_from(value: f32) -> Result<Self, Self::Error> {
        Ok(Self(value.to_be_bytes()))
    }
}
impl From<Float> for f32 {
    #[inline(always)]
    fn from(value: Float) -> Self {
        f32::from_be_bytes(value.0)
    }
}

impl<'s> TryFrom<&'s str> for String<'s> {
    type Error = InvalidContents;
    #[inline(always)]
    fn try_from(value: &'s str) -> Result<Self, Self::Error> {
        if !value.is_ascii() {
            Err(InvalidContents::NonAscii)
        } else if value.contains('\0') {
            Err(InvalidContents::NullInString)
        } else {
            Ok(Self(value))
        }
    }
}
impl<'s> From<String<'s>> for &'s str {
    #[inline(always)]
    fn from(value: String<'s>) -> Self {
        value.0
    }
}

impl<'b> TryFrom<&'b [u8]> for Blob<'b> {
    type Error = core::convert::Infallible;
    #[inline(always)]
    fn try_from(value: &'b [u8]) -> Result<Self, Self::Error> {
        Ok(Self(value))
    }
}
impl<'b, const N: usize> TryFrom<&'b [u8; N]> for Blob<'b> {
    type Error = core::convert::Infallible;
    #[inline(always)]
    fn try_from(value: &'b [u8; N]) -> Result<Self, Self::Error> {
        Ok(Self(value))
    }
}
impl<'b> From<Blob<'b>> for &'b [u8] {
    #[inline(always)]
    fn from(value: Blob<'b>) -> Self {
        value.0
    }
}

#[cfg(feature = "alloc")]
impl TryFrom<alloc::string::String> for DynamicString {
    type Error = InvalidContents;
    #[inline(always)]
    fn try_from(value: alloc::string::String) -> Result<Self, Self::Error> {
        if !value.is_ascii() {
            Err(InvalidContents::NonAscii)
        } else if value.contains('\0') {
            Err(InvalidContents::NullInString)
        } else {
            Ok(Self(value))
        }
    }
}
#[cfg(feature = "alloc")]
impl From<DynamicString> for alloc::string::String {
    #[inline(always)]
    fn from(value: DynamicString) -> Self {
        value.0
    }
}

#[cfg(feature = "alloc")]
#[allow(unused_qualifications)]
impl TryFrom<alloc::vec::Vec<u8>> for DynamicBlob {
    type Error = core::convert::Infallible;
    #[inline(always)]
    fn try_from(value: alloc::vec::Vec<u8>) -> Result<Self, Self::Error> {
        Ok(Self(value))
    }
}
#[cfg(feature = "alloc")]
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

#[cfg(feature = "alloc")]
impl IntoIterator for Data {
    type IntoIter = Batched<<Self as Atomic>::Iter>;
    type Item = u8;
    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        #[allow(unused_qualifications)]
        let v: alloc::vec::Vec<_> = match self {
            Data::Integer(i) => i.into_iter().collect(),
            Data::Float(f) => f.into_iter().collect(),
            Data::String(s) => s.into_iter().collect(),
            Data::Blob(b) => b.into_iter().collect(),
        };
        v.into_iter().batch()
    }
}

#[cfg(feature = "alloc")]
impl IntoIterator for DynamicString {
    type IntoIter = Batched<<Self as Atomic>::Iter>;
    type Item = u8;
    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_bytes().into_iter().chain(once(0)).batch()
    }
}

#[cfg(feature = "alloc")]
impl IntoIterator for DynamicBlob {
    type IntoIter = Batched<<Self as Atomic>::Iter>;
    type Item = u8;
    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter().batch()
    }
}

//////////////// `Decode` implementations

impl Decode for Integer {
    type Error = core::convert::Infallible;
    #[inline(always)]
    fn decode<I: Iterator<Item = u8>>(iter: &mut I) -> Result<Self, Misaligned4B<Self::Error>> {
        Aligned4B::decode(iter).map(|Aligned4B(a, b, c, d, _)| Self([a, b, c, d]))
    }
}

impl Decode for Float {
    type Error = core::convert::Infallible;
    #[inline(always)]
    fn decode<I: Iterator<Item = u8>>(iter: &mut I) -> Result<Self, Misaligned4B<Self::Error>> {
        Aligned4B::decode(iter).map(|Aligned4B(a, b, c, d, _)| Self([a, b, c, d]))
    }
}

#[non_exhaustive]
#[cfg(feature = "alloc")]
/// Any possible error while decoding an OSC string.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum StringDecodeErr {
    /// Not an ASCII character.
    NonAscii(u8),
    /// Returned a null terminator then the rest of the 4-byte chunk was not null.
    NullThenNonNull,
}

#[cfg(feature = "alloc")]
impl core::fmt::Display for StringDecodeErr {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            &Self::NonAscii(c) => {
                write!(
                    f,
                    "Matched a non-ASCII character in an alleged OSC string: '{}'.",
                    core::ascii::escape_default(c)
                )
            }
            &Self::NullThenNonNull => write!(
                f,
                "Matched a string's null terminator, \
                but the following padding bytes were non-null.",
            ),
        }
    }
}

#[cfg(feature = "alloc")]
impl Decode for DynamicString {
    type Error = StringDecodeErr;
    #[inline]
    fn decode<I: Iterator<Item = u8>>(iter: &mut I) -> Result<Self, Misaligned4B<Self::Error>> {
        let mut s = alloc::string::String::new();
        loop {
            let bytes = Aligned4B::decode(iter)?;
            if bytes.0 == b'\0' {
                if bytes.1 != b'\0' || bytes.2 != b'\0' || bytes.3 != b'\0' {
                    return Err(Misaligned4B::Other(StringDecodeErr::NullThenNonNull));
                }
                return Ok(Self(s));
            }
            if !bytes.0.is_ascii() {
                return Err(Misaligned4B::Other(StringDecodeErr::NonAscii(bytes.0)));
            }
            s.push(char::from(bytes.0));
            if bytes.1 == b'\0' {
                if bytes.2 != b'\0' || bytes.3 != b'\0' {
                    return Err(Misaligned4B::Other(StringDecodeErr::NullThenNonNull));
                }
                return Ok(Self(s));
            }
            if !bytes.1.is_ascii() {
                return Err(Misaligned4B::Other(StringDecodeErr::NonAscii(bytes.1)));
            }
            s.push(char::from(bytes.1));
            if bytes.2 == b'\0' {
                if bytes.3 != b'\0' {
                    return Err(Misaligned4B::Other(StringDecodeErr::NullThenNonNull));
                }
                return Ok(Self(s));
            }
            if !bytes.2.is_ascii() {
                return Err(Misaligned4B::Other(StringDecodeErr::NonAscii(bytes.2)));
            }
            s.push(char::from(bytes.2));
            if bytes.3 == b'\0' {
                return Ok(Self(s));
            }
            if !bytes.3.is_ascii() {
                return Err(Misaligned4B::Other(StringDecodeErr::NonAscii(bytes.3)));
            }
            s.push(char::from(bytes.3));
        }
    }
}

#[non_exhaustive]
#[cfg(feature = "alloc")]
/// Any possible error while decoding an OSC blob.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum BlobDecodeErr {
    /// The first bit of the OSC size is null, almost surely a misinterpretation.
    NegativeSize,
    /// Expected a null padding byte but found non-null.
    TooLong,
    /// Returned a null terminator then the rest of the 4-byte chunk was not null.
    NullThenNonNull,
}

#[cfg(feature = "alloc")]
impl core::fmt::Display for BlobDecodeErr {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            &Self::NegativeSize => write!(
                f,
                "OSC blob size is negative. \
                This is almost surely a result of an earlier error that \
                offset bytes and interpreted something else as the size."
            ),
            &Self::TooLong => write!(
                f,
                "OSC blob longer than claimed: \
                expected a null padding byte but found non-null."
            ),
            &Self::NullThenNonNull => write!(
                f,
                "Matched a string's null terminator, \
                but the following padding bytes were non-null.",
            ),
        }
    }
}

#[cfg(feature = "alloc")]
impl Decode for DynamicBlob {
    type Error = BlobDecodeErr;
    #[inline]
    fn decode<I: Iterator<Item = u8>>(iter: &mut I) -> Result<Self, Misaligned4B<Self::Error>> {
        #[allow(unsafe_code)]
        // SAFETY:
        // Infallible. Checked at compile time.
        let size: u32 = i32::from(unsafe { Integer::decode(iter).unwrap_unchecked() })
            .try_into()
            .or(Err(Misaligned4B::Other(BlobDecodeErr::NegativeSize)))?;
        #[allow(clippy::default_numeric_fallback)]
        let chunks = size >> 3;
        let mut v = alloc::vec::Vec::with_capacity(chunks.try_into().unwrap_or(0));
        for _ in 0..chunks {
            let bytes = Aligned4B::decode(iter)?;
            v.push(bytes.0);
            v.push(bytes.1);
            v.push(bytes.2);
            v.push(bytes.3);
        }
        Ok(Self(v))
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

    #[cfg(feature = "alloc")]
    impl IntoAtomic for crate::Data {}
    #[allow(unused_qualifications)]
    #[cfg(feature = "alloc")]
    impl IntoAtomic for alloc::string::String {}
    #[allow(unused_qualifications)]
    #[cfg(feature = "alloc")]
    impl IntoAtomic for alloc::vec::Vec<u8> {}
}

/// Rust types that map 1-to-1 to atomic OSC types.
#[allow(clippy::module_name_repetitions)]
pub trait IntoAtomic: sealed::IntoAtomic + IntoOsc + Sized
where
    InvalidContents: From<<Self::AsAtomic as TryFrom<Self>>::Error>,
{
    /// The OSC type that directly corresponds to this Rust type.
    type AsAtomic: Atomic<AsRust = Self>;
    /// Convert directly into the OSC representation of this Rust type.
    /// # Errors
    /// If the data is invalid (e.g. a non-ASCII string).
    #[inline(always)]
    fn into_atomic(self) -> Result<Self::AsAtomic, <Self::AsAtomic as TryFrom<Self>>::Error> {
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

#[cfg(feature = "alloc")]
impl IntoAtomic for Data {
    type AsAtomic = Data;
}

#[cfg(feature = "alloc")]
impl IntoAtomic for alloc::string::String {
    type AsAtomic = DynamicString;
}

#[cfg(feature = "alloc")]
#[allow(unused_qualifications)]
impl IntoAtomic for alloc::vec::Vec<u8> {
    type AsAtomic = DynamicBlob;
}

//////////////// QuickCheck implementations

#[cfg(feature = "quickcheck")]
#[allow(clippy::unwrap_used, unused_qualifications)]
mod prop {
    //! Implementations of `quickcheck::Arbitrary`.

    use super::*;

    impl quickcheck::Arbitrary for Integer {
        #[inline]
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            i32::arbitrary(g).into_atomic().unwrap()
        }
        #[inline]
        fn shrink(&self) -> alloc::boxed::Box<dyn Iterator<Item = Self>> {
            alloc::boxed::Box::new(
                self.into_rust()
                    .shrink()
                    .filter_map(|e| e.into_atomic().ok()),
            )
        }
    }

    impl quickcheck::Arbitrary for Float {
        #[inline]
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            f32::arbitrary(g).into_atomic().unwrap()
        }
        #[inline]
        fn shrink(&self) -> alloc::boxed::Box<dyn Iterator<Item = Self>> {
            alloc::boxed::Box::new(
                self.into_rust()
                    .shrink()
                    .filter_map(|e| e.into_atomic().ok()),
            )
        }
    }

    impl quickcheck::Arbitrary for DynamicString {
        #[inline]
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            let mut s = alloc::string::String::arbitrary(g);
            s.retain(|c| c.is_ascii() && c != '\0');
            s.into_atomic().unwrap()
        }
        #[inline]
        fn shrink(&self) -> alloc::boxed::Box<dyn Iterator<Item = Self>> {
            alloc::boxed::Box::new(
                self.clone()
                    .into_rust()
                    .shrink()
                    .filter_map(|e| e.into_atomic().ok()),
            )
        }
    }

    #[allow(unused_qualifications)]
    impl quickcheck::Arbitrary for DynamicBlob {
        #[inline]
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            alloc::vec::Vec::arbitrary(g).into_atomic().unwrap()
        }
        #[inline]
        fn shrink(&self) -> alloc::boxed::Box<dyn Iterator<Item = Self>> {
            alloc::boxed::Box::new(
                self.clone()
                    .into_rust()
                    .shrink()
                    .filter_map(|e| e.into_atomic().ok()),
            )
        }
    }
}
