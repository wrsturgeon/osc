/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Single-character OSC type tags.

/// Any possible error in decoding an unknown number of OSC type tags.
#[non_exhaustive]
#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum TagDecodeErr {
    /// Unrecognized type tag character.
    UnrecognizedTypeTag(u8),
    /// Missing a comma to demarcate the beginning of type tags.
    MissingComma(u8),
    /// Returned a null terminator then the rest of the 4-byte chunk was not null.
    NullThenNonNull,
}

impl core::fmt::Display for TagDecodeErr {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            &Self::UnrecognizedTypeTag(c) => write!(
                f,
                "Unrecognized type tag character: '{}'",
                core::ascii::escape_default(c)
            ),
            &Self::MissingComma(c) => write!(
                f,
                "Expected a comma to begin type tags but got '{}'",
                core::ascii::escape_default(c)
            ),
            &Self::NullThenNonNull => write!(
                f,
                "OSC address returned a null terminator, \
                but then the rest of its 4-byte chunk was non-null."
            ),
        }
    }
}

/// Single-character OSC type tag.
#[repr(u8)]
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Tag {
    /// 32-bit big-endian signed two's-complement integer.
    Integer = b'i',
    /// 32-bit big-endian IEEE 754 floating-point number.
    Float = b'f',
    /// Null-terminated (not your responsibility!) byte string.
    String = b's',
    /// Arbitrary known-length collection of bytes.
    Blob = b'b',
}

impl TryFrom<u8> for Tag {
    type Error = TagDecodeErr;
    #[inline(always)]
    #[allow(unused_variables)] // TODO: remove
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            b'i' => Self::Integer,
            b'f' => Self::Float,
            b's' => Self::String,
            b'b' => Self::Blob,
            _ => return Err(TagDecodeErr::UnrecognizedTypeTag(value)),
        })
    }
}

#[cfg(feature = "quickcheck")]
impl quickcheck::Arbitrary for Tag {
    #[inline]
    #[allow(clippy::unwrap_used)]
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        *g.choose(&[Tag::Integer, Tag::Float, Tag::String, Tag::Blob])
            .unwrap()
    }
    #[inline]
    #[allow(unused_qualifications)]
    fn shrink(&self) -> alloc::boxed::Box<dyn Iterator<Item = Self>> {
        alloc::boxed::Box::new([Self::Integer, Self::Float, Self::String, Self::Blob].into_iter())
    }
}
