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
pub trait IntoAddress: Sized + IntoIterator
where
    Self::Item: IntoIntoAddress,
    Self::IntoIter: Clone,
    <Self::Item as IntoIntoAddress>::IntoAddr: Clone,
{
    /// Fuse a list of strings into an OSC address by interspersing with `/`.
    /// # Errors
    /// If the address is invalid (according to the OSC spec).
    #[inline(always)]
    #[allow(clippy::type_complexity)]
    fn into_address(
        self,
    ) -> Result<
        Address<
            core::iter::Map<
                Self::IntoIter,
                fn(Self::Item) -> <Self::Item as IntoIntoAddress>::IntoAddr,
            >,
        >,
        AddressErr,
    > {
        #[allow(clippy::as_conversions, clippy::as_underscore, trivial_casts)]
        let iter = self
            .into_iter()
            .map(<Self::Item as IntoIntoAddress>::into_into_addr as _);
        if iter.clone().next().is_none() {
            return Err(AddressErr::Empty);
        }
        for s in iter.clone() {
            for byte in s {
                if !valid_address_character(byte) {
                    return Err(AddressErr::InvalidCharacter(byte));
                }
            }
        }
        Ok(Address::new(iter))
    }
}

impl<I: IntoIterator> IntoAddress for I
where
    I::Item: IntoIntoAddress,
    I::IntoIter: Clone,
    <I::Item as IntoIntoAddress>::IntoAddr: Clone,
{
}

/// An OSC address, e.g. `/lighting/right/...`
#[derive(Clone)]
#[allow(missing_debug_implementations)]
pub struct Address<I: Iterator>
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

impl<I: Iterator> Address<I>
where
    I::Item: IntoIterator<Item = u8>,
    <I::Item as IntoIterator>::IntoIter: Clone,
{
    /// Initialize a new address given an iterator over strings.
    #[inline]
    pub const fn new(iter: I) -> Self {
        Self {
            iter,
            bytes: None,
            slash: true,
        }
    }
}

impl<I: Iterator + Default> Default for Address<I>
where
    I::Item: IntoIterator<Item = u8>,
    <I::Item as IntoIterator>::IntoIter: Clone,
{
    #[inline]
    fn default() -> Self {
        Self::new(I::default())
    }
}

impl<I: Iterator> Iterator for Address<I>
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

#[allow(clippy::module_name_repetitions)]
#[cfg(any(test, feature = "quickcheck"))]
pub type QCAddress =
    Address<core::iter::Map<alloc::vec::IntoIter<String>, fn(String) -> alloc::vec::IntoIter<u8>>>;

#[allow(clippy::unwrap_used)]
#[cfg(any(test, feature = "quickcheck"))]
impl quickcheck::Arbitrary for QCAddress {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        let mut vv: Vec<String> = alloc::vec::Vec::arbitrary(g);
        for v in &mut vv {
            v.retain(|c| u8::try_from(c).map_or(false, valid_address_character));
        }
        vv.into_address().unwrap()
    }
    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(
            self.iter
                .clone()
                .map(|v| String::from_utf8(v.collect()).unwrap())
                .collect::<Vec<_>>()
                .shrink()
                .map(|mut vv| {
                    for v in &mut vv {
                        v.retain(|c| u8::try_from(c).map_or(false, valid_address_character));
                    }
                    vv.into_address().unwrap()
                }),
        )
    }
}
