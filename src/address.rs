/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use core::str::Bytes;

/// Error in an OSC address.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum AddressErr {
    /// Empty iterator, i.e. no path segments.
    Empty,
    /// Invalid characters in addresses (defined in the OSC spec).
    InvalidCharacter(u8),
}

/// Fuse a list of strings into an OSC address by interspersing with `/`.
pub trait IntoAddress<'s>: Sized + IntoIterator<Item = &'s str>
where
    Self::IntoIter: Clone,
{
    /// Fuse a list of strings into an OSC address by interspersing with `/`.
    #[inline(always)]
    fn into_address(self) -> Result<Address<'s, Self::IntoIter>, AddressErr> {
        let iter = self.into_iter();
        if iter.clone().next().is_none() {
            return Err(AddressErr::Empty);
        }
        for s in iter.clone() {
            for byte in s.bytes() {
                match byte {
                    b' ' | b'#' | b'*' | b',' | b'/' | b'?' | b'[' | b']' | b'{' | b'}' => {
                        return Err(AddressErr::InvalidCharacter(byte))
                    }
                    _ => {}
                }
            }
        }
        Ok(Address::new(iter))
    }
}

impl<'s, I: IntoIterator<Item = &'s str>> IntoAddress<'s> for I where I::IntoIter: Clone {}

/// An OSC address, e.g. `/lighting/right/...`
#[derive(Clone, Debug)]
pub struct Address<'s, I: Iterator<Item = &'s str>> {
    iter: I,
    bytes: Option<Bytes<'s>>,
    slash: bool,
}

impl<'s, I: Iterator<Item = &'s str>> Address<'s, I> {
    /// Initialize a new address given an iterator over strings.
    pub const fn new(iter: I) -> Self {
        Self {
            iter,
            bytes: None,
            slash: true,
        }
    }
}

impl<'s, I: Iterator<Item = &'s str> + Default> Default for Address<'s, I> {
    fn default() -> Self {
        Self::new(I::default())
    }
}

impl<'s, I: Iterator<Item = &'s str>> Iterator for Address<'s, I> {
    type Item = u8;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            return if self.slash {
                self.slash = false;
                let (bytes, ch) = self
                    .iter
                    .next()
                    .map_or_else(|| (None, b'\0'), |s| (Some(s.bytes()), b'/'));
                self.bytes = bytes;
                Some(ch)
            } else {
                let some @ Some(_) = self.bytes.as_mut()?.next() else {
                    self.slash = true;
                    continue;
                };
                some
            };
        }
    }
}
