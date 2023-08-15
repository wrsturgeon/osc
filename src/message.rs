/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Typed data to a specified address.

use crate::{address::Address, tuple::Tuple, AddressErr, Batch, Batched, IntoAddress};
use core::iter::{once, Chain, Once};

/// Typed data to a specified address.
#[repr(transparent)]
#[derive(Clone, Debug)]
#[allow(clippy::type_complexity)]
pub struct Message<'a, A: Iterator<Item = &'a str> + Clone, T: Tuple>(
    Chain<
        Chain<Batched<Address<'a, A>>, Batched<Chain<Chain<Once<u8>, T::TypeTagIter>, Once<u8>>>>,
        T::Chained,
    >,
);

impl<'a, A: Iterator<Item = &'a str> + Clone, T: Tuple> Message<'a, A, T> {
    /// Prefer `.into_osc()`, but if you already have OSC data, this is fine.
    /// # Errors
    /// If the address is invalid (according to the OSC spec).
    #[inline]
    pub fn new<I: IntoAddress<'a, IntoIter = A>>(address: I, data: T) -> Result<Self, AddressErr> {
        Ok(Self(
            address
                .into_address()?
                .batch()
                .chain(once(b',').chain(data.type_tag()).chain(once(b'\0')).batch())
                .chain(data.chain()),
        ))
    }
}

impl<'a, A: Iterator<Item = &'a str> + Clone, T: Tuple> Iterator for Message<'a, A, T> {
    type Item = u8;
    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

// TODO:
// #[cfg(any(test, feature = "quickcheck"))]
// impl<'a, A: Iterator<Item = &'a str> + Clone, T: Tuple> quickcheck::Arbitrary
//     for Message<'a, A, T>
// {
// }
