/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Format a Rust type as an OSC message.

use crate::{
    address::IntoIntoAddress, tuple::Tuple, AddressErr, Blob, Dynamic, DynamicBlob, DynamicString,
    Float, Integer, IntoAddress, IntoAtomic, Message, String,
};

/// Format a Rust type as an OSC message.
pub trait IntoOsc {
    /// OSC equivalent of this Rust type.
    type AsOsc: Tuple;
    /// Format a Rust type as an OSC message.
    /// # Errors
    /// If the address is invalid (according to the OSC spec).
    #[allow(clippy::type_complexity)]
    fn into_osc<I: IntoAddress>(self, address: I) -> Result<Message<I, Self::AsOsc>, AddressErr>
    where
        I::Item: IntoIntoAddress,
        <I::Item as IntoIntoAddress>::IntoAddr: Clone;
}

impl IntoOsc for i32 {
    type AsOsc = (Integer,);
    #[inline(always)]
    fn into_osc<I: IntoAddress>(self, address: I) -> Result<Message<I, Self::AsOsc>, AddressErr>
    where
        I::Item: IntoIntoAddress,
        <I::Item as IntoIntoAddress>::IntoAddr: Clone,
    {
        Ok(Message::new(address.into_address()?, (self.into_atomic(),)))
    }
}

impl IntoOsc for f32 {
    type AsOsc = (Float,);
    #[inline(always)]
    fn into_osc<I: IntoAddress>(self, address: I) -> Result<Message<I, Self::AsOsc>, AddressErr>
    where
        I::Item: IntoIntoAddress,
        <I::Item as IntoIntoAddress>::IntoAddr: Clone,
    {
        Ok(Message::new(address.into_address()?, (self.into_atomic(),)))
    }
}

impl<'s> IntoOsc for &'s str {
    type AsOsc = (String<'s>,);
    #[inline(always)]
    fn into_osc<I: IntoAddress>(self, address: I) -> Result<Message<I, Self::AsOsc>, AddressErr>
    where
        I::Item: IntoIntoAddress,
        <I::Item as IntoIntoAddress>::IntoAddr: Clone,
    {
        Ok(Message::new(address.into_address()?, (self.into_atomic(),)))
    }
}

impl<'b> IntoOsc for &'b [u8] {
    type AsOsc = (Blob<'b>,);
    #[inline(always)]
    fn into_osc<I: IntoAddress>(self, address: I) -> Result<Message<I, Self::AsOsc>, AddressErr>
    where
        I::Item: IntoIntoAddress,
        <I::Item as IntoIntoAddress>::IntoAddr: Clone,
    {
        Ok(Message::new(address.into_address()?, (self.into_atomic(),)))
    }
}

impl IntoOsc for () {
    type AsOsc = ();
    #[inline(always)]
    fn into_osc<I: IntoAddress>(self, address: I) -> Result<Message<I, Self::AsOsc>, AddressErr>
    where
        I::Item: IntoIntoAddress,
        <I::Item as IntoIntoAddress>::IntoAddr: Clone,
    {
        Ok(Message::new(address.into_address()?, ()))
    }
}

/// Implement `IntoOsc` for a tuple of types, each of which implement `IntoAtomic`.
macro_rules! impl_for_tuple {
    ($($id:ident),+) => {
        impl<$($id: IntoAtomic),+> IntoOsc for ($($id),+,) {
            type AsOsc = ($($id::AsAtomic),+,);
            #[inline(always)]
            #[allow(non_snake_case)]
            fn into_osc<I: IntoAddress>(
                self,
                address: I,
            ) -> Result<Message<I, Self::AsOsc>, AddressErr>
            where
                I::Item: IntoIntoAddress,
                <I::Item as IntoIntoAddress>::IntoAddr: Clone,
            {
                let ($($id),+,) = self;
                Ok(Message::new(address.into_address()?, ($($id.into_atomic()),+,)))
            }
        }
    };
}

impl_for_tuple!(A);
impl_for_tuple!(A, B);
impl_for_tuple!(A, B, C);
impl_for_tuple!(A, B, C, D);
impl_for_tuple!(A, B, C, D, E);
impl_for_tuple!(A, B, C, D, E, F);
impl_for_tuple!(A, B, C, D, E, F, G);
impl_for_tuple!(A, B, C, D, E, F, G, H);

#[cfg(feature = "alloc")]
impl IntoOsc for Dynamic {
    type AsOsc = (Dynamic,);
    #[inline(always)]
    fn into_osc<I: IntoAddress>(self, address: I) -> Result<Message<I, Self::AsOsc>, AddressErr>
    where
        I::Item: IntoIntoAddress,
        <I::Item as IntoIntoAddress>::IntoAddr: Clone,
    {
        Ok(Message::new(address.into_address()?, (self,)))
    }
}

#[cfg(feature = "alloc")]
impl IntoOsc for alloc::string::String {
    type AsOsc = (DynamicString,);
    #[inline(always)]
    fn into_osc<I: IntoAddress>(self, address: I) -> Result<Message<I, Self::AsOsc>, AddressErr>
    where
        I::Item: IntoIntoAddress,
        <I::Item as IntoIntoAddress>::IntoAddr: Clone,
    {
        Ok(Message::new(address.into_address()?, (self.into_atomic(),)))
    }
}

#[cfg(feature = "alloc")]
#[allow(unused_qualifications)]
impl IntoOsc for alloc::vec::Vec<u8> {
    type AsOsc = (DynamicBlob,);
    #[inline(always)]
    fn into_osc<I: IntoAddress>(self, address: I) -> Result<Message<I, Self::AsOsc>, AddressErr>
    where
        I::Item: IntoIntoAddress,
        <I::Item as IntoIntoAddress>::IntoAddr: Clone,
    {
        Ok(Message::new(address.into_address()?, (self.into_atomic(),)))
    }
}
