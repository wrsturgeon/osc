/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! OSC values whose types can't be known at compile time.

use crate::{DynamicBlob, DynamicString, Float, Integer};

/// OSC values whose types can't be known at compile time.
#[non_exhaustive]
#[cfg(feature = "alloc")]
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Dynamic {
    /// 32-bit big-endian signed two's-complement integer.
    Integer(Integer),
    /// 32-bit big-endian IEEE 754 floating-point number.
    Float(Float),
    /// Null-terminated (not your responsibility!) byte string.
    String(DynamicString),
    /// Arbitrary known-length collection of bytes.
    Blob(DynamicBlob),
}

impl TryFrom<Dynamic> for Integer {
    type Error = Dynamic;
    #[inline(always)]
    fn try_from(value: Dynamic) -> Result<Self, Self::Error> {
        if let Dynamic::Integer(v) = value {
            Ok(v)
        } else {
            Err(value)
        }
    }
}

impl TryFrom<Dynamic> for Float {
    type Error = Dynamic;
    #[inline(always)]
    fn try_from(value: Dynamic) -> Result<Self, Self::Error> {
        if let Dynamic::Float(v) = value {
            Ok(v)
        } else {
            Err(value)
        }
    }
}

impl TryFrom<Dynamic> for DynamicString {
    type Error = Dynamic;
    #[inline(always)]
    fn try_from(value: Dynamic) -> Result<Self, Self::Error> {
        if let Dynamic::String(v) = value {
            Ok(v)
        } else {
            Err(value)
        }
    }
}

impl TryFrom<Dynamic> for DynamicBlob {
    type Error = Dynamic;
    #[inline(always)]
    fn try_from(value: Dynamic) -> Result<Self, Self::Error> {
        if let Dynamic::Blob(v) = value {
            Ok(v)
        } else {
            Err(value)
        }
    }
}

#[cfg(feature = "quickcheck")]
#[allow(unused_qualifications)]
impl quickcheck::Arbitrary for Dynamic {
    #[inline]
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        #[allow(
            clippy::as_conversions,
            clippy::as_underscore,
            clippy::shadow_unrelated,
            trivial_casts
        )]
        let opt = g.choose(&[
            (|g| Self::Integer(Integer::arbitrary(g))) as fn(_) -> _,
            (|g| Self::Float(Float::arbitrary(g))) as _,
            (|g| Self::String(DynamicString::arbitrary(g))) as _,
            (|g| Self::Blob(DynamicBlob::arbitrary(g))) as _,
        ]);
        #[allow(unsafe_code)]
        // SAFETY:
        // QuickCheck guarantees that a non-empty slice will yield `Some(_)`
        let f = unsafe { opt.unwrap_unchecked() };
        f(g)
    }
    #[inline]
    fn shrink(&self) -> alloc::boxed::Box<dyn Iterator<Item = Self>> {
        match self {
            &Self::Integer(ref i) => alloc::boxed::Box::new(i.shrink().map(Self::Integer)),
            &Self::Float(ref f) => alloc::boxed::Box::new(f.shrink().map(Self::Float)),
            &Self::String(ref s) => alloc::boxed::Box::new(s.shrink().map(Self::String)),
            &Self::Blob(ref b) => alloc::boxed::Box::new(b.shrink().map(Self::Blob)),
        }
    }
}
