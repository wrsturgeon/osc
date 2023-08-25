/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! An OSC address, e.g. `/lighting/right/...`

use crate::{Batch, Batched, InvalidContents};

#[cfg(feature = "alloc")]
use crate::{Aligned4B, Decode, Misaligned4B};

/// Error in an OSC address.
#[non_exhaustive]
#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum AddressErr {
    /// Empty iterator, i.e. no path segments.
    Empty,
    /// Invalid characters in addresses (defined in the OSC spec).
    InvalidCharacter(u8),
    /// Invalid string contents for conversion to OSC.
    StringErr(InvalidContents),
}

/// Convert from this type into an iterator in a specified way.
#[allow(clippy::module_name_repetitions)]
pub trait IntoIntoAddress: Clone {
    /// Iterator converted from this type in a specified way.
    type IntoAddr: Iterator<Item = u8>;
    /// Convert from this type into an iterator in a specified way.
    fn into_into_addr(self) -> Self::IntoAddr;
}

impl<'s> IntoIntoAddress for &'s str {
    type IntoAddr = core::str::Bytes<'s>;
    #[inline(always)]
    fn into_into_addr(self) -> Self::IntoAddr {
        self.bytes()
    }
}

impl<'s> IntoIntoAddress for &'_ &'s str {
    type IntoAddr = core::str::Bytes<'s>;
    #[inline(always)]
    fn into_into_addr(self) -> Self::IntoAddr {
        self.bytes()
    }
}

#[cfg(feature = "alloc")]
#[allow(unused_qualifications)]
impl IntoIntoAddress for alloc::string::String {
    type IntoAddr = alloc::vec::IntoIter<u8>;
    #[inline(always)]
    fn into_into_addr(self) -> Self::IntoAddr {
        self.into_bytes().into_iter()
    }
}

/// Is this a valid ASCII character that's not blacklisted in the OSC spec?
#[inline]
#[must_use]
pub const fn valid_address_character(c: u8) -> bool {
    match c {
        b' ' | b'#' | b'*' | b',' | b'/' | b'?' | b'[' | b']' | b'{' | b'}' => false,
        32..=126 => true, // "printable" ASCII characters, per the spec
        _ => false,
    }
}

/// Fuse a list of strings into an OSC address by interspersing with `/`.
#[allow(clippy::module_name_repetitions)]
pub trait IntoAddress<Method: IntoIntoAddress>:
    Sized + Clone + IntoIterator<Item = Method>
{
    /// Fuse a list of strings into an OSC address by interspersing with `/`.
    /// # Errors
    /// If the address is invalid (according to the OSC spec).
    #[inline(always)]
    #[allow(clippy::type_complexity)]
    fn into_address(self, method: Method) -> Result<Address<Self, Method>, AddressErr> {
        #[allow(clippy::as_conversions, clippy::as_underscore, trivial_casts)]
        let iter = self
            .clone()
            .into_iter()
            .map(IntoIntoAddress::into_into_addr);
        for mut s in iter {
            match s.next() {
                None => return Err(AddressErr::Empty),
                Some(c) => {
                    if !valid_address_character(c) {
                        return Err(AddressErr::InvalidCharacter(c));
                    }
                }
            }
            for c in s {
                if !valid_address_character(c) {
                    return Err(AddressErr::InvalidCharacter(c));
                }
            }
        }
        let mut m = method.clone().into_into_addr();
        match m.next() {
            None => return Err(AddressErr::Empty),
            Some(c) => {
                if !valid_address_character(c) {
                    return Err(AddressErr::InvalidCharacter(c));
                }
            }
        }
        for c in m {
            if !valid_address_character(c) {
                return Err(AddressErr::InvalidCharacter(c));
            }
        }
        Ok(Address(self, method))
    }
}

impl<I: Clone + IntoIterator> IntoAddress<I::Item> for I where I::Item: IntoIntoAddress {}

/// An OSC address, e.g. `/lighting/right/...`
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Address<Path: IntoIterator<Item = Method>, Method: IntoIntoAddress>(
    pub(crate) Path,
    pub(crate) Method,
);

impl<Path: IntoIterator<Item = Method>, Method: IntoIntoAddress> IntoIterator
    for Address<Path, Method>
{
    type Item = u8;
    type IntoIter = Batched<
        Iter<
            core::iter::Chain<
                core::iter::Map<Path::IntoIter, fn(Method) -> Method::IntoAddr>,
                core::iter::Once<Method::IntoAddr>,
            >,
        >,
    >;
    #[inline]
    #[allow(clippy::as_conversions, clippy::as_underscore, trivial_casts)]
    fn into_iter(self) -> Self::IntoIter {
        Iter {
            iter: self
                .0
                .into_iter()
                .map(IntoIntoAddress::into_into_addr as _)
                .chain(core::iter::once(self.1.into_into_addr())),
            bytes: None,
            slash: true,
        }
        .batch()
    }
}

/// Iterator over an OSC address, e.g. `/lighting/right/...`
#[allow(missing_debug_implementations)]
pub struct Iter<I: Iterator>
where
    I::Item: IntoIterator<Item = u8>,
{
    /// Iterator over path segments.
    iter: I,
    /// Iterator over bytes in only the current path segment.
    bytes: Option<<I::Item as IntoIterator>::IntoIter>,
    /// Whether the next item should be a slash.
    slash: bool,
}

impl<I: Iterator> Iterator for Iter<I>
where
    I::Item: IntoIterator<Item = u8>,
{
    type Item = u8;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            return if self.slash {
                self.slash = false;
                let (bytes, ch) = self
                    .iter
                    .next()
                    .map_or_else(|| (None, b'\0'), |s| (Some(s.into_iter()), b'/'));
                self.bytes = bytes;
                Some(ch)
            } else {
                let some @ Some(_) = self.bytes.as_mut()?.next() else {
                    self.slash = true;
                    continue;
                };
                #[allow(clippy::let_and_return)]
                some
            };
        }
    }
}

#[cfg(feature = "quickcheck")]
#[allow(clippy::unwrap_used, unused_qualifications)]
impl quickcheck::Arbitrary
    for Address<alloc::vec::Vec<alloc::string::String>, alloc::string::String>
{
    #[inline]
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        let mut vv: alloc::vec::Vec<alloc::string::String> = alloc::vec::Vec::arbitrary(g);
        for v in &mut vv {
            v.retain(|c| u8::try_from(c).map_or(false, valid_address_character));
        }
        vv.retain(|v| !v.is_empty());
        loop {
            let mut s = alloc::string::String::arbitrary(g);
            s.retain(|c| u8::try_from(c).map_or(false, valid_address_character));
            if s.is_empty() {
                continue;
            }
            return vv.into_address(s).unwrap();
        }
    }
    #[inline]
    fn shrink(&self) -> alloc::boxed::Box<dyn Iterator<Item = Self>> {
        alloc::boxed::Box::new(
            (self.0.clone(), self.1.clone())
                .shrink()
                .filter_map(|(vv, m)| vv.into_address(m).ok()),
        )
    }
}

/// Error encountered while decoding an OSC address.
#[non_exhaustive]
#[cfg(feature = "alloc")]
#[allow(clippy::module_name_repetitions)]
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

#[cfg(feature = "alloc")]
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
        let mut first = Aligned4B::decode(iter)?.into_iter();
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
            let bytes = Aligned4B::decode(iter)?;
            match parse_address_chars(bytes, &mut post_slash, &mut v) {
                None => {}
                Some(Ok(head)) => return Ok(Address(v, head)),
                Some(Err(e)) => return Err(e),
            }
        }
    }
}
