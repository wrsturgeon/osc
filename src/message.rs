/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Typed data to a specified address.

use crate::{
    address::{Address, IntoIntoAddress},
    tuple::Tuple,
    Batch, Batched,
};
use core::iter::{once, Chain, Once};

/// Default type parameter for the address of a message.
#[cfg(feature = "alloc")]
#[allow(unused_qualifications)]
type AddrDefault = alloc::vec::Vec<alloc::string::String>;
/// Default type parameter for the data of a message.
#[cfg(feature = "alloc")]
#[allow(unused_qualifications)]
type DataDefault = alloc::vec::Vec<crate::Dynamic>;

/// Default type parameter for the address of a message.
#[cfg(not(feature = "alloc"))]
type AddrDefault = &'static [&'static str];
/// Default type parameter for the data of a message.
#[cfg(not(feature = "alloc"))]
type DataDefault = ();

/// OSC message: address, type tag (inferred), and data.
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Message<Addr: IntoIterator = AddrDefault, Data: Tuple = DataDefault>
where
    Addr::Item: IntoIntoAddress,
    <Addr::Item as IntoIntoAddress>::IntoAddr: Clone,
{
    /// Address without type tag.
    address: Address<Addr>,
    /// Data after address & type tag.
    data: Data,
}

impl<Addr: IntoIterator, Data: Tuple> Message<Addr, Data>
where
    Addr::Item: IntoIntoAddress,
    <Addr::Item as IntoIntoAddress>::IntoAddr: Clone,
{
    /// Prefer `.into_osc()`, but if you already have OSC data, this is fine.
    /// # Errors
    /// If the address is invalid (according to the OSC spec).
    #[inline]
    pub const fn new(address: Address<Addr>, data: Data) -> Self {
        Self { address, data }
    }
}

impl<Addr: IntoIterator, Data: Tuple> IntoIterator for Message<Addr, Data>
where
    Addr::Item: IntoIntoAddress,
    <Addr::Item as IntoIntoAddress>::IntoAddr: Clone,
{
    type Item = u8;
    type IntoIter = Chain<
        Chain<
            Batched<<Address<Addr> as IntoIterator>::IntoIter>,
            Batched<Chain<Chain<Once<u8>, Data::TypeTagIter>, Once<u8>>>,
        >,
        Data::Chained,
    >;
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.address
            .into_iter()
            .batch()
            .chain(
                once(b',')
                    .chain(self.data.type_tag())
                    .chain(once(b'\0'))
                    .batch(),
            )
            .chain(self.data.chain())
    }
}

#[allow(unused_qualifications)]
#[cfg(feature = "quickcheck")]
impl quickcheck::Arbitrary
    for Message<alloc::vec::Vec<alloc::string::String>, alloc::vec::Vec<crate::Dynamic>>
{
    #[inline]
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        Message::new(Address::arbitrary(g), alloc::vec::Vec::arbitrary(g))
    }
    #[inline]
    fn shrink(&self) -> alloc::boxed::Box<dyn Iterator<Item = Self>> {
        alloc::boxed::Box::new(
            self.address
                .shrink()
                .zip(self.data.shrink())
                .map(|(address, data)| Self { address, data }),
        )
    }
}
