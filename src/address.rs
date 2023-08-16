/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! An OSC address, e.g. `/lighting/right/...`

/// Error in an OSC address.
#[non_exhaustive]
#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum AddressErr {
    /// Empty iterator, i.e. no path segments.
    Empty,
    /// Invalid characters in addresses (defined in the OSC spec).
    InvalidCharacter(u8),
}

/// Convert from this type into an iterator in a specified way.
#[allow(clippy::module_name_repetitions)]
pub trait IntoIntoAddress {
    /// Iterator converted from this type in a specified way.
    type IntoAddr: Iterator<Item = u8>;
    /// Convert from this type into an iterator in a specified way.
    fn into_into_addr(self) -> Self::IntoAddr;
}

impl<'s> IntoIntoAddress for &'s str {
    type IntoAddr = core::str::Bytes<'s>;
    fn into_into_addr(self) -> Self::IntoAddr {
        self.bytes()
    }
}

#[allow(unused_qualifications)]
impl IntoIntoAddress for alloc::string::String {
    type IntoAddr = alloc::vec::IntoIter<u8>;
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
        _ => c.is_ascii(),
    }
}

/// Fuse a list of strings into an OSC address by interspersing with `/`.
#[allow(clippy::module_name_repetitions)]
pub trait IntoAddress: Sized + Clone + IntoIterator
where
    Self::Item: IntoIntoAddress,
    <Self::Item as IntoIntoAddress>::IntoAddr: Clone,
{
    /// Fuse a list of strings into an OSC address by interspersing with `/`.
    /// # Errors
    /// If the address is invalid (according to the OSC spec).
    #[inline(always)]
    #[allow(clippy::type_complexity)]
    fn into_address(self) -> Result<Address<Self>, AddressErr> {
        #[allow(clippy::as_conversions, clippy::as_underscore, trivial_casts)]
        let mut iter = self
            .clone()
            .into_iter()
            .map(IntoIntoAddress::into_into_addr);
        match iter.next() {
            None => return Err(AddressErr::Empty),
            Some(s) => {
                for c in s {
                    if !valid_address_character(c) {
                        return Err(AddressErr::InvalidCharacter(c));
                    }
                }
            }
        }
        for s in iter {
            for c in s {
                if !valid_address_character(c) {
                    return Err(AddressErr::InvalidCharacter(c));
                }
            }
        }
        Ok(Address(self))
    }
}

impl<I: Clone + IntoIterator> IntoAddress for I
where
    I::Item: IntoIntoAddress,
    <I::Item as IntoIntoAddress>::IntoAddr: Clone,
{
}

/// An OSC address, e.g. `/lighting/right/...`
#[repr(transparent)]
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Address<I: IntoIterator>(I)
where
    I::Item: IntoIntoAddress,
    <I::Item as IntoIntoAddress>::IntoAddr: Clone;

impl<I: IntoIterator> IntoIterator for Address<I>
where
    I::Item: IntoIntoAddress,
    <I::Item as IntoIntoAddress>::IntoAddr: Clone,
{
    type Item = u8;
    type IntoIter =
        Iter<core::iter::Map<I::IntoIter, fn(I::Item) -> <I::Item as IntoIntoAddress>::IntoAddr>>;
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        Iter {
            iter: self.0.into_iter().map(IntoIntoAddress::into_into_addr),
            bytes: None,
            slash: true,
        }
    }
}

/// Iterator over an OSC address, e.g. `/lighting/right/...`
#[derive(Clone)]
#[allow(missing_debug_implementations)]
pub struct Iter<I: Iterator>
where
    I::Item: IntoIterator<Item = u8>,
    <I::Item as IntoIterator>::IntoIter: Clone,
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
    <I::Item as IntoIterator>::IntoIter: Clone,
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

#[allow(clippy::unwrap_used, unused_qualifications)]
#[cfg(feature = "quickcheck")]
impl quickcheck::Arbitrary for Address<alloc::vec::Vec<alloc::string::String>> {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        loop {
            let mut vv: alloc::vec::Vec<alloc::string::String> = alloc::vec::Vec::arbitrary(g);
            if vv.is_empty() {
                continue;
            }
            for v in &mut vv {
                v.retain(|c| u8::try_from(c).map_or(false, valid_address_character));
            }
            return vv.into_address().unwrap();
        }
    }
    fn shrink(&self) -> alloc::boxed::Box<dyn Iterator<Item = Self>> {
        alloc::boxed::Box::new(self.0.shrink().map(|mut vv| {
            for v in &mut vv {
                v.retain(|c| u8::try_from(c).map_or(false, valid_address_character));
            }
            vv.into_address().unwrap()
        }))
    }
}
