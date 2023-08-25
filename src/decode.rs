/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Read a stream of bytes into this OSC type or provide a reason we couldn't.

use core::marker::PhantomData;

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
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Misaligned4B<E> {
    /// Ended when we expected more input.
    End,
    /// Number of bytes was not a multiple of 4.
    Misaligned,
    /// Number of bytes was a multiple of 4, but another error occurred.
    Other(E),
}

// impl<A, B: From<A>> From<Misaligned4B<A>> for Misaligned4B<B> {
//     fn from(value: Misaligned4B<A>) -> Self {
//         match value {
//             Misaligned4B::End => Misaligned4B::End,
//             Misaligned4B::Misaligned => Misaligned4B::Misaligned,
//             Misaligned4B::Other(o) => Misaligned4B::Other(o.into()),
//         }
//     }
// }

/// Four bytes read at the same time.
/// Idea is that length should always be a multiple of 4.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Aligned4B<E: core::fmt::Display>(
    pub u8,
    pub u8,
    pub u8,
    pub u8,
    pub(crate) PhantomData<E>,
);

impl<E: core::fmt::Display> IntoIterator for Aligned4B<E> {
    type Item = u8;
    type IntoIter = core::array::IntoIter<u8, 4>;
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        [self.0, self.1, self.2, self.3].into_iter()
    }
}

impl<E: core::fmt::Display> Decode for Aligned4B<E> {
    type Error = E;
    #[inline]
    fn decode<I: Iterator<Item = u8>>(iter: &mut I) -> Result<Self, Misaligned4B<Self::Error>> {
        Ok(Self(
            iter.next().ok_or(Misaligned4B::End)?,
            iter.next().ok_or(Misaligned4B::Misaligned)?,
            iter.next().ok_or(Misaligned4B::Misaligned)?,
            iter.next().ok_or(Misaligned4B::Misaligned)?,
            PhantomData,
        ))
    }
}
