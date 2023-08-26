/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! OSC values whose types can't be known at compile time.

use crate::{
    Aligned4B, Batch, Batched, Decode, DynamicBlob, DynamicString, Float, Integer, Misaligned4B,
    Tag, TagDecodeErr,
};

/// Unknown number of OSC type tags.
#[repr(transparent)]
#[allow(unused_qualifications)]
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Tags(pub(crate) alloc::vec::Vec<Tag>);

impl IntoIterator for Tags {
    type Item = u8;
    type IntoIter = Batched<
        core::iter::Chain<
            core::iter::Chain<
                core::iter::Once<u8>,
                core::iter::Map<alloc::vec::IntoIter<Tag>, fn(Tag) -> u8>,
            >,
            core::iter::Once<u8>,
        >,
    >;
    #[inline]
    #[allow(clippy::as_conversions, clippy::as_underscore, trivial_casts)]
    fn into_iter(self) -> Self::IntoIter {
        core::iter::once(b',')
            .chain(self.0.into_iter().map((|c: Tag| c as u8) as _))
            .chain(core::iter::once(b'\0'))
            .batch()
    }
}

#[allow(unused_qualifications)]
impl Decode for Tags {
    type Error = TagDecodeErr;
    #[inline]
    fn decode<I: Iterator<Item = u8>>(iter: &mut I) -> Result<Self, Misaligned4B<Self::Error>> {
        let mut v = alloc::vec![];
        {
            let first = Aligned4B::decode(iter)?;
            if first.0 != b',' {
                return Err(Misaligned4B::Other(TagDecodeErr::MissingComma(first.0)));
            }
            if first.1 == b'\0' {
                if first.2 != b'\0' || first.3 != b'\0' {
                    return Err(Misaligned4B::Other(TagDecodeErr::NullThenNonNull));
                }
                return Ok(Self(v));
            }
            v.push(first.1.try_into().map_err(Misaligned4B::Other)?);
            if first.2 == b'\0' {
                if first.3 != b'\0' {
                    return Err(Misaligned4B::Other(TagDecodeErr::NullThenNonNull));
                }
                return Ok(Self(v));
            }
            v.push(first.2.try_into().map_err(Misaligned4B::Other)?);
            if first.3 == b'\0' {
                return Ok(Self(v));
            }
            v.push(first.3.try_into().map_err(Misaligned4B::Other)?);
        }
        loop {
            let bytes = Aligned4B::decode(iter)?;
            if bytes.0 == b'\0' {
                if bytes.1 != b'\0' || bytes.2 != b'\0' || bytes.3 != b'\0' {
                    return Err(Misaligned4B::Other(TagDecodeErr::NullThenNonNull));
                }
                return Ok(Self(v));
            }
            v.push(bytes.0.try_into().map_err(Misaligned4B::Other)?);
            if bytes.1 == b'\0' {
                if bytes.2 != b'\0' || bytes.3 != b'\0' {
                    return Err(Misaligned4B::Other(TagDecodeErr::NullThenNonNull));
                }
                return Ok(Self(v));
            }
            v.push(bytes.1.try_into().map_err(Misaligned4B::Other)?);
            if bytes.2 == b'\0' {
                if bytes.3 != b'\0' {
                    return Err(Misaligned4B::Other(TagDecodeErr::NullThenNonNull));
                }
                return Ok(Self(v));
            }
            v.push(bytes.2.try_into().map_err(Misaligned4B::Other)?);
            if bytes.3 == b'\0' {
                return Ok(Self(v));
            }
            v.push(bytes.3.try_into().map_err(Misaligned4B::Other)?);
        }
    }
}

#[cfg(feature = "quickcheck")]
impl quickcheck::Arbitrary for Tags {
    #[inline]
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        Self(quickcheck::Arbitrary::arbitrary(g))
    }
    #[inline]
    #[allow(unused_qualifications)]
    fn shrink(&self) -> alloc::boxed::Box<dyn Iterator<Item = Self>> {
        alloc::boxed::Box::new(self.0.shrink().map(Self))
    }
}

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

impl core::fmt::Display for DynamicDecodeErr {
    #[inline(always)]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            &DynamicDecodeErr::TypeTagErr(e) => write!(f, "{e}"),
        }
    }
}

impl From<TagDecodeErr> for DynamicDecodeErr {
    #[inline]
    fn from(value: TagDecodeErr) -> Self {
        Self::TypeTagErr(value)
    }
}

#[allow(unused_qualifications)]
impl Decode for Dynamic {
    type Error = DynamicDecodeErr;
    #[inline]
    fn decode<I: Iterator<Item = u8>>(iter: &mut I) -> Result<Self, Misaligned4B<Self::Error>> {
        let types = match Tags::decode(iter) {
            Ok(ok) => ok,
            Err(Misaligned4B::End) => return Err(Misaligned4B::End),
            Err(Misaligned4B::Misaligned) => return Err(Misaligned4B::Misaligned),
            Err(Misaligned4B::Other(o)) => {
                return Err(Misaligned4B::Other(DynamicDecodeErr::TypeTagErr(o)))
            }
        };
        let mut v = alloc::vec::Vec::with_capacity(types.0.len());
        for tag in types.0 {
            #[allow(unsafe_code, unused_unsafe)]
            // TODO:
            // SAFETY:
            // Uncertain. Revisit after property testing.
            v.push(unsafe {
                match tag {
                    Tag::Integer => Data::Integer(Integer::decode(iter).unwrap_unchecked()),
                    Tag::Float => Data::Float(Float::decode(iter).unwrap_unchecked()),
                    Tag::String => Data::String(DynamicString::decode(iter).unwrap_unchecked()),
                    Tag::Blob => Data::Blob(DynamicBlob::decode(iter).unwrap_unchecked()),
                }
            });
        }
        Ok(Self(v))
    }
}

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

#[cfg(feature = "quickcheck")]
#[allow(unused_qualifications)]
impl quickcheck::Arbitrary for Dynamic {
    #[inline]
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        Self(quickcheck::Arbitrary::arbitrary(g))
    }
    #[inline]
    fn shrink(&self) -> alloc::boxed::Box<dyn Iterator<Item = Self>> {
        alloc::boxed::Box::new(self.0.shrink().map(Self))
    }
}
