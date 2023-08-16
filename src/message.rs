/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Typed data to a specified address.

use crate::{address::Address, tuple::Tuple, Batch, Batched};
use core::iter::{once, Chain, Once};

/// Typed data to a specified address.
#[repr(transparent)]
#[derive(Clone)]
#[allow(clippy::type_complexity, missing_debug_implementations)]
pub struct Message<A: Iterator, T: Tuple>(
    Chain<
        Chain<Batched<Address<A>>, Batched<Chain<Chain<Once<u8>, T::TypeTagIter>, Once<u8>>>>,
        T::Chained,
    >,
)
where
    A::Item: Clone + IntoIterator<Item = u8>,
    <A::Item as IntoIterator>::IntoIter: Clone;

impl<A: Iterator, T: Tuple> Message<A, T>
where
    A::Item: Clone + Iterator<Item = u8>,
{
    /// Prefer `.into_osc()`, but if you already have OSC data, this is fine.
    /// # Errors
    /// If the address is invalid (according to the OSC spec).
    #[inline]
    pub fn new(address: Address<A>, data: T) -> Self {
        Self(
            address
                .batch()
                .chain(once(b',').chain(data.type_tag()).chain(once(b'\0')).batch())
                .chain(data.chain()),
        )
    }
}

impl<A: Iterator, T: Tuple> Iterator for Message<A, T>
where
    A::Item: Iterator<Item = u8>,
    <A::Item as IntoIterator>::IntoIter: Clone,
{
    type Item = u8;
    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

#[allow(unused_qualifications)]
#[cfg(any(test, feature = "quickcheck"))]
impl quickcheck::Arbitrary
    for Message<
        core::iter::Map<
            alloc::vec::IntoIter<alloc::string::String>,
            fn(alloc::string::String) -> alloc::vec::IntoIter<u8>,
        >,
        alloc::vec::Vec<crate::Dynamic>,
    >
{
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        use crate::IntoAddress;
        // TODO: explicitly avoid `Err` instead of just resampling (could take a while on long addresses)
        loop {
            let r = alloc::vec::Vec::<alloc::string::String>::arbitrary(g).into_address();
            let Ok(addr) = r else { continue; };
            return Message::new(addr, alloc::vec::Vec::arbitrary(g));
        }
    }
}
