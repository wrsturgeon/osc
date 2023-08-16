/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Read a stream of bytes into this OSC type or provide a reason we couldn't.

#[cfg(feature = "alloc")]
use crate::Address;

/// Read a stream of bytes into this OSC type or provide a reason we couldn't.
pub trait Decode: Sized {
    /// Reasons this might fail.
    type Error: core::fmt::Display;
    /// Read a stream of bytes into this OSC type or provide a reason we couldn't.
    /// # Errors
    /// If the stream's length is not a multiple of 4 or if we encounter any issues along the way.
    fn decode<I: Iterator<Item = u8>>(iter: &mut I) -> Result<Self, Misaligned4B<Self::Error>>;
}

/// Anywhere we could read a number of bytes not a multiple of four.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Misaligned4B<E> {
    /// Number of bytes was not a multiple of 4.
    Misaligned,
    /// Number of bytes was a multiple of 4, but another error occurred.
    Other(E),
}

/// Four bytes read at the same time.
/// Idea is that length should always be a multiple of 4.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Aligned4B(u8, u8, u8, u8);

impl IntoIterator for Aligned4B {
    type Item = u8;
    type IntoIter = core::array::IntoIter<u8, 4>;
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        [self.0, self.1, self.2, self.3].into_iter()
    }
}

impl Decode for Aligned4B {
    type Error = core::convert::Infallible;
    #[inline]
    fn decode<I: Iterator<Item = u8>>(iter: &mut I) -> Result<Self, Misaligned4B<Self::Error>> {
        Ok(Self(
            iter.next().ok_or(Misaligned4B::Misaligned)?,
            iter.next().ok_or(Misaligned4B::Misaligned)?,
            iter.next().ok_or(Misaligned4B::Misaligned)?,
            iter.next().ok_or(Misaligned4B::Misaligned)?,
        ))
    }
}

/// Error encountered while decoding an OSC address.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum AddressDecodeErr {
    /// Missing the leading `/`.
    LeadingSlash {
        /// Character seen instead of `/`.
        actual: u8,
    },
    /// Ends after a `/`.
    NoMethod,
    /// Immediate `//` with nothing in between.
    EmptySegment,
    /// Blacklisted character, e.g. `*`.
    PatternsNotYetImplemented(u8),
    /// Not a printable ASCII character.
    NotPrintableAscii(u8),
    /// Returned a null terminator then the rest of the 4-byte chunk was not null.
    NullThenNonNull,
}

impl core::fmt::Display for AddressDecodeErr {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            &Self::LeadingSlash { actual } => write!(
                f,
                "OSC address missing leading slash (got '{}' instead).",
                core::ascii::escape_default(actual)
            ),
            &Self::NoMethod => write!(
                f,
                "OSC address missing a method, i.e. ending immediately after a '/'."
            ),
            &Self::EmptySegment => write!(
                f,
                "OSC address with a zero-sized segment, i.e. back-to-back slashes \"//\"."
            ),
            &Self::PatternsNotYetImplemented(c) => write!(
                f,
                "OSC pattern matching not yet implemented (required for the character '{}'). \
                (Want to help? Open a pull request with your implementation!)",
                core::ascii::escape_default(c)
            ),
            &Self::NotPrintableAscii(c) => write!(
                f,
                "OSC address should be entirely printable ASCII characters (got '{}')",
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

/// Parse an individual character with some mutable state passed in.
#[inline]
#[cfg(feature = "alloc")]
#[allow(unsafe_code, unused_qualifications)]
fn parse_address_char<I: Iterator<Item = u8>>(
    byte: u8,
    bytes: &mut I,
    post_slash: &mut bool,
    v: &mut alloc::vec::Vec<alloc::string::String>,
) -> Option<Result<alloc::string::String, Misaligned4B<AddressDecodeErr>>> {
    match byte {
        b'\0' => {
            if *post_slash {
                Some(Err(Misaligned4B::Other(AddressDecodeErr::NoMethod)))
            } else {
                for c in bytes {
                    if c != b'\0' {
                        return Some(Err(Misaligned4B::Other(AddressDecodeErr::NullThenNonNull)));
                    }
                }
                // SAFETY:
                // Control flow guarantees this will not be empty.
                Some(Ok(unsafe { v.pop().unwrap_unchecked() }))
            }
        }
        b'/' => {
            if *post_slash {
                Some(Err(Misaligned4B::Other(AddressDecodeErr::EmptySegment)))
            } else {
                v.push(alloc::string::String::new());
                None
            }
        }
        c @ (b' ' | b'#' | b'*' | b',' | b'?' | b'[' | b']' | b'{' | b'}') => Some(Err(
            Misaligned4B::Other(AddressDecodeErr::PatternsNotYetImplemented(c)),
        )),
        c @ (..=31 | 127..) => Some(Err(Misaligned4B::Other(
            AddressDecodeErr::NotPrintableAscii(c),
        ))),
        c => {
            *post_slash = false;
            // SAFETY:
            // Control flow guarantees this will not be empty.
            unsafe { v.last_mut().unwrap_unchecked() }.push(char::from(c));
            None
        }
    }
}

/// Parse four individual characters with some mutable state passed in.
#[inline]
#[cfg(feature = "alloc")]
#[allow(unsafe_code, unused_qualifications)]
fn parse_address_chars<I: IntoIterator<Item = u8>>(
    bytes: I,
    post_slash: &mut bool,
    v: &mut alloc::vec::Vec<alloc::string::String>,
) -> Option<Result<alloc::string::String, Misaligned4B<AddressDecodeErr>>> {
    let mut iter = bytes.into_iter();
    while let Some(byte) = iter.next() {
        if let some @ Some(_) = parse_address_char(byte, &mut iter, post_slash, v) {
            return some;
        }
    }
    None
}

#[cfg(feature = "alloc")]
#[allow(unused_qualifications)]
impl Decode for Address<alloc::vec::Vec<alloc::string::String>, alloc::string::String> {
    type Error = AddressDecodeErr;
    #[inline]
    #[allow(unsafe_code)]
    fn decode<I: Iterator<Item = u8>>(iter: &mut I) -> Result<Self, Misaligned4B<Self::Error>> {
        let mut first = Aligned4B::decode(iter)
            .or(Err(Misaligned4B::Misaligned))?
            .into_iter();
        // SAFETY:
        // Control flow guarantees this will not be empty.
        let actual = unsafe { first.next().unwrap_unchecked() };
        if actual != b'/' {
            return Err(Misaligned4B::Other(AddressDecodeErr::LeadingSlash {
                actual,
            }));
        }
        let mut post_slash = true;
        let mut v = alloc::vec![alloc::string::String::new()];
        match parse_address_char(
            // SAFETY:
            // Control flow guarantees this will not be empty.
            unsafe { first.next().unwrap_unchecked() },
            &mut first,
            &mut post_slash,
            &mut v,
        ) {
            None => {}
            Some(Ok(head)) => return Ok(Address(v, head)),
            Some(Err(e)) => return Err(e),
        };
        match parse_address_chars(&mut first, &mut post_slash, &mut v) {
            None => {}
            Some(Ok(head)) => return Ok(Address(v, head)),
            Some(Err(e)) => return Err(e),
        }
        loop {
            let bytes = Aligned4B::decode(iter).or(Err(Misaligned4B::Misaligned))?;
            match parse_address_chars(bytes, &mut post_slash, &mut v) {
                None => {}
                Some(Ok(head)) => return Ok(Address(v, head)),
                Some(Err(e)) => return Err(e),
            }
        }
    }
}
