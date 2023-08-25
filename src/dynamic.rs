/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! OSC values whose types can't be known at compile time.

use crate::{DynamicBlob, DynamicString, Float, Integer, Tag};

/// Unknown number of OSC type tags.
#[repr(transparent)]
#[allow(unused_qualifications)]
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Tags(alloc::vec::Vec<Tag>);

/// Any possible error in decoding an unknown number of OSC type tags.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum TagDecodeErr {
    /// Returned a null terminator then the rest of the 4-byte chunk was not null.
    NullThenNonNull,
}

impl core::fmt::Display for TagDecodeErr {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            &Self::NullThenNonNull => write!(
                f,
                "OSC address returned a null terminator, \
                but then the rest of its 4-byte chunk was non-null."
            ),
        }
    }
}

// TODO:
// #[allow(unused_qualifications)]
// impl Decode for Tags {
//     type Error = TagDecodeErr;
//     #[inline]
//     fn decode<I: Iterator<Item = u8>>(iter: &mut I) -> Result<Self, Misaligned4B<Self::Error>> {
//         let mut first = Aligned4B::decode(iter)?;
//         todo!()
//     }
// }

/// OSC values whose types can't be known at compile time.
#[non_exhaustive]
#[cfg(feature = "alloc")]
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Data {
    /// 32-bit big-endian signed two's-complement integer.
    Integer(Integer),
    /// 32-bit big-endian IEEE 754 floating-point number.
    Float(Float),
    /// Null-terminated (not your responsibility!) byte string.
    String(DynamicString),
    /// Arbitrary known-length collection of bytes.
    Blob(DynamicBlob),
}

impl TryFrom<Data> for Integer {
    type Error = Data;
    #[inline(always)]
    fn try_from(value: Data) -> Result<Self, Self::Error> {
        if let Data::Integer(v) = value {
            Ok(v)
        } else {
            Err(value)
        }
    }
}

impl TryFrom<Data> for Float {
    type Error = Data;
    #[inline(always)]
    fn try_from(value: Data) -> Result<Self, Self::Error> {
        if let Data::Float(v) = value {
            Ok(v)
        } else {
            Err(value)
        }
    }
}

impl TryFrom<Data> for DynamicString {
    type Error = Data;
    #[inline(always)]
    fn try_from(value: Data) -> Result<Self, Self::Error> {
        if let Data::String(v) = value {
            Ok(v)
        } else {
            Err(value)
        }
    }
}

impl TryFrom<Data> for DynamicBlob {
    type Error = Data;
    #[inline(always)]
    fn try_from(value: Data) -> Result<Self, Self::Error> {
        if let Data::Blob(v) = value {
            Ok(v)
        } else {
            Err(value)
        }
    }
}

/// Vector of data whose types are unknown at compile time.
#[repr(transparent)]
#[allow(unused_qualifications)]
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Dynamic(pub(crate) alloc::vec::Vec<Data>);

/// Any possible errors while parsing an OSC message of unknown structure.
#[non_exhaustive]
#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DynamicDecodeErr {
    /// Error parsing type tags.
    TypeTagErr(TagDecodeErr),
}

// impl core::fmt::Display for DynamicDecodeErr {
//     #[inline(always)]
//     fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
//         match self {}
//     }
// }

// impl From<TagDecodeErr> for DynamicDecodeErr {
//     #[inline]
//     fn from(value: TagDecodeErr) -> Self {
//         Self::TypeTagErr(value)
//     }
// }

// TODO:
// #[allow(unused_qualifications)]
// impl Decode for Dynamic {
//     type Error = DynamicDecodeErr;
//     #[inline]
//     fn decode<I: Iterator<Item = u8>>(iter: &mut I) -> Result<Self, Misaligned4B<Self::Error>> {
//         let types = match Tags::decode(iter) {
//             Ok(ok) => ok,
//             Err(Misaligned4B::End) => return Err(Misaligned4B::End),
//             Err(Misaligned4B::Misaligned) => return Err(Misaligned4B::Misaligned),
//             Err(Misaligned4B::Other(o)) => {
//                 return Err(Misaligned4B::Other(DynamicDecodeErr::TypeTagErr(o)))
//             }
//         };
//         let mut v = alloc::vec::Vec::with_capacity(types.0.len());
//         for tag in types.0 {
//             #[allow(unsafe_code, unused_unsafe)]
//             v.push(match tag {
//                 Tag::Integer => Data::Integer(unsafe { Integer::decode(iter).unwrap_unchecked() }),
//             });
//         }
//         Ok(Self(v))
//     }
// }

#[cfg(feature = "quickcheck")]
#[allow(unused_qualifications)]
impl quickcheck::Arbitrary for Data {
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
