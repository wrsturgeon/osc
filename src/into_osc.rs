/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::{
    tuple::Tuple, AddressErr, Blob, Float, Integer, IntoAddress, IntoAtomic, Message, String,
};

/// Format a Rust type as an OSC message.
pub trait IntoOsc {
    /// OSC equivalent of this Rust type.
    type AsOsc: Tuple;
    /// Format a Rust type as an OSC message.
    fn into_osc<'a, I: IntoAddress<'a>>(
        self,
        address: I,
    ) -> Result<Message<'a, I::IntoIter, Self::AsOsc>, AddressErr>
    where
        I::IntoIter: Clone;
}

impl IntoOsc for i32 {
    type AsOsc = (Integer,);
    #[inline(always)]
    fn into_osc<'a, I: IntoAddress<'a>>(
        self,
        address: I,
    ) -> Result<Message<'a, I::IntoIter, Self::AsOsc>, AddressErr>
    where
        I::IntoIter: Clone,
    {
        Message::new(address, (self.into_atomic(),))
    }
}

impl IntoOsc for f32 {
    type AsOsc = (Float,);
    #[inline(always)]
    fn into_osc<'a, I: IntoAddress<'a>>(
        self,
        address: I,
    ) -> Result<Message<'a, I::IntoIter, Self::AsOsc>, AddressErr>
    where
        I::IntoIter: Clone,
    {
        Message::new(address, (self.into_atomic(),))
    }
}

impl<'s> IntoOsc for &'s str {
    type AsOsc = (String<'s>,);
    #[inline(always)]
    fn into_osc<'a, I: IntoAddress<'a>>(
        self,
        address: I,
    ) -> Result<Message<'a, I::IntoIter, Self::AsOsc>, AddressErr>
    where
        I::IntoIter: Clone,
    {
        Message::new(address, (self.into_atomic(),))
    }
}

impl<'b> IntoOsc for &'b [u8] {
    type AsOsc = (Blob<'b>,);
    #[inline(always)]
    fn into_osc<'a, I: IntoAddress<'a>>(
        self,
        address: I,
    ) -> Result<Message<'a, I::IntoIter, Self::AsOsc>, AddressErr>
    where
        I::IntoIter: Clone,
    {
        Message::new(address, (self.into_atomic(),))
    }
}

impl IntoOsc for () {
    type AsOsc = ();
    #[inline(always)]
    fn into_osc<'a, I: IntoAddress<'a>>(
        self,
        address: I,
    ) -> Result<Message<'a, I::IntoIter, Self::AsOsc>, AddressErr>
    where
        I::IntoIter: Clone,
    {
        Message::new(address, ())
    }
}

macro_rules! impl_for_tuple {
    ($($id:ident),+) => {
        impl<$($id: IntoAtomic),+> IntoOsc for ($($id),+,) {
            type AsOsc = ($($id::AsAtomic),+,);
            #[inline(always)]
            #[allow(non_snake_case)]
            fn into_osc<'a, I: IntoAddress<'a>>(
                self,
                address: I,
            ) -> Result<Message<'a, I::IntoIter,  Self::AsOsc>, AddressErr>
            where
                I::IntoIter: Clone,
            {
                let ($($id),+,) = self;
                Message::new(address, ($($id.into_atomic()),+,))
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
