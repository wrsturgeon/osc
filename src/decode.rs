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
    type Error;
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
    InvalidCharacter(u8),
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
        let mut s = alloc::string::String::new();
        // SAFETY:
        // Control flow guarantees this will not be empty.
        match unsafe { first.next().unwrap_unchecked() } {
            b'\0' => return Err(Misaligned4B::Other(AddressDecodeErr::NoMethod)),
            b'/' => return Err(Misaligned4B::Other(AddressDecodeErr::EmptySegment)),
            c @ (b' ' | b'#' | b'*' | b',' | b'?' | b'[' | b']' | b'{' | b'}') => {
                return Err(Misaligned4B::Other(AddressDecodeErr::InvalidCharacter(c)))
            }
            c => s.push(char::from(c)),
        };
        let mut post_slash = false;
        let mut v = alloc::vec![s];
        for byte in first {
            match byte {
                b'\0' => {
                    if post_slash {
                        return Err(Misaligned4B::Other(AddressDecodeErr::NoMethod));
                    }
                    // SAFETY:
                    // Control flow guarantees this will not be empty.
                    let head = unsafe { v.pop().unwrap_unchecked() };
                    return Ok(Address(v, head));
                }
                b'/' => {
                    if post_slash {
                        return Err(Misaligned4B::Other(AddressDecodeErr::EmptySegment));
                    }
                    post_slash = false;
                    v.push(alloc::string::String::new());
                }
                c @ (b' ' | b'#' | b'*' | b',' | b'?' | b'[' | b']' | b'{' | b'}') => {
                    return Err(Misaligned4B::Other(AddressDecodeErr::InvalidCharacter(c)));
                }
                c => {
                    post_slash = false;
                    // SAFETY:
                    // Control flow guarantees this will not be empty.
                    unsafe { v.last_mut().unwrap_unchecked() }.push(char::from(c));
                }
            }
        }
        loop {
            let bytes = Aligned4B::decode(iter).or(Err(Misaligned4B::Misaligned))?;
            for byte in bytes {
                match byte {
                    b'\0' => {
                        if post_slash {
                            return Err(Misaligned4B::Other(AddressDecodeErr::NoMethod));
                        }
                        // SAFETY:
                        // Control flow guarantees this will not be empty.
                        let head = unsafe { v.pop().unwrap_unchecked() };
                        return Ok(Address(v, head));
                    }
                    b'/' => {
                        if post_slash {
                            return Err(Misaligned4B::Other(AddressDecodeErr::EmptySegment));
                        }
                        post_slash = false;
                        v.push(alloc::string::String::new());
                    }
                    c @ (b' ' | b'#' | b'*' | b',' | b'?' | b'[' | b']' | b'{' | b'}') => {
                        return Err(Misaligned4B::Other(AddressDecodeErr::InvalidCharacter(c)));
                    }
                    c @ (0..=31 | 127..) => {
                        return Err(Misaligned4B::Other(AddressDecodeErr::InvalidCharacter(c)))
                    }
                    c => {
                        post_slash = false;
                        // SAFETY:
                        // Control flow guarantees this will not be empty.
                        unsafe { v.last_mut().unwrap_unchecked() }.push(char::from(c));
                    }
                }
            }
        }
    }
}
