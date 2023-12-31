/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Typed data to a specified address.

use crate::{
    address::{Address, IntoIntoAddress},
    tuple::Tuple,
    Batch, Batched, Tag,
};
use core::iter::{once, Chain, Once};

/// Default type parameter for the path of a message.
#[cfg(feature = "alloc")]
#[allow(unused_qualifications)]
type AddrDefault = alloc::vec::Vec<MethodDefault>;
/// Default type parameter for the method of a message.
#[cfg(feature = "alloc")]
#[allow(unused_qualifications)]
type MethodDefault = alloc::string::String;
/// Default type parameter for the data of a message.
#[cfg(feature = "alloc")]
#[allow(unused_qualifications)]
type DataDefault = crate::Dynamic;

/// Default type parameter for the path of a message.
#[cfg(not(feature = "alloc"))]
type AddrDefault = &'static [MethodDefault];
/// Default type parameter for the method of a message.
#[cfg(not(feature = "alloc"))]
type MethodDefault = &'static str;
/// Default type parameter for the data of a message.
#[cfg(not(feature = "alloc"))]
type DataDefault = ();

/// OSC message: address, type tag (inferred), and data.
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Message<
    Path: IntoIterator<Item = Method> = AddrDefault,
    Method: IntoIntoAddress = MethodDefault,
    Data: Tuple = DataDefault,
> {
    /// Address without type tag.
    address: Address<Path, Method>,
    /// Data after address & type tag.
    data: Data,
}

impl<Path: IntoIterator<Item = Method>, Method: IntoIntoAddress, Data: Tuple>
    Message<Path, Method, Data>
{
    /// Prefer `.into_osc()`, but if you already have OSC data, this is fine.
    /// # Errors
    /// If the address is invalid (according to the OSC spec).
    #[inline]
    pub const fn new(address: Address<Path, Method>, data: Data) -> Self {
        Self { address, data }
    }
}

impl<Path: IntoIterator<Item = Method>, Method: IntoIntoAddress, Data: Tuple> IntoIterator
    for Message<Path, Method, Data>
{
    type Item = u8;
    type IntoIter = Chain<
        Chain<
            <Address<Path, Method> as IntoIterator>::IntoIter,
            Batched<
                Chain<Chain<Once<u8>, core::iter::Map<Data::TypeTagIter, fn(Tag) -> u8>>, Once<u8>>,
            >,
        >,
        Data::Chained,
    >;
    #[inline]
    #[allow(clippy::as_conversions, clippy::as_underscore, trivial_casts)]
    fn into_iter(self) -> Self::IntoIter {
        self.address
            .into_iter()
            // batched already
            .chain(
                once(b',')
                    .chain(self.data.type_tag().map((|tag| tag as _) as fn(Tag) -> u8))
                    .chain(once(b'\0'))
                    .batch(),
            )
            .chain(self.data.chain())
    }
}

#[allow(unused_qualifications)]
#[cfg(feature = "quickcheck")]
impl quickcheck::Arbitrary for Message {
    #[inline]
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        Message::new(
            Address::arbitrary(g),
            crate::Dynamic(alloc::vec::Vec::arbitrary(g)),
        )
    }
    #[inline]
    fn shrink(&self) -> alloc::boxed::Box<dyn Iterator<Item = Self>> {
        alloc::boxed::Box::new(self.address.shrink().zip(self.data.0.shrink()).map(
            |(address, data)| Self {
                address,
                data: crate::Dynamic(data),
            },
        ))
    }
}
