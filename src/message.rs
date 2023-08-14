/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Typed data to a specified address.

use crate::{tuple::Tuple, Batch, Batched, String};
use core::iter::{once, Chain, Once};

/// Typed data to a specified address.
#[repr(transparent)]
#[derive(Clone, Debug)]
pub struct Message<'a, T: Tuple>(
    Chain<
        Chain<
            Batched<Chain<Once<u8>, Chain<core::str::Bytes<'a>, Once<u8>>>>,
            Batched<Chain<Once<u8>, T::TypeTagIter>>,
        >,
        T::Chained,
    >,
);

impl<'a, T: Tuple> Message<'a, T> {
    /// Prefer `.into_osc()`, but if you already have OSC data, this is fine.
    pub fn new(address: String<'a>, data: T) -> Self {
        debug_assert!(address.is_address());
        Self(
            once(b'/')
                .chain(address.into_iter().unbatch())
                .batch()
                .chain(T::type_tag())
                .chain(data.chain()),
        )
    }
}

impl<T: Tuple> Iterator for Message<'_, T> {
    type Item = u8;
    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}
