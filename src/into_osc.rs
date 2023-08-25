/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Format a Rust type as an OSC message.

use crate::{
    AddressErr, Blob, Float, Integer, IntoAddress, IntoAtomic, IntoIntoAddress, InvalidContents,
    Message, String, Tuple,
};

#[cfg(feature = "alloc")]
use crate::{Data, Dynamic, DynamicBlob, DynamicString};

/// Format a Rust type as an OSC message.
pub trait IntoOsc {
    /// OSC equivalent of this Rust type.
    type AsOsc: Tuple;
    /// Format a Rust type as an OSC message.
    /// # Errors
    /// If the address is invalid (according to the OSC spec).
    #[allow(clippy::type_complexity)]
    fn into_osc<Path: IntoAddress<Method>, Method: IntoIntoAddress>(
        self,
        path: Path,
        method: Method,
    ) -> Result<Message<Path, Method, Self::AsOsc>, AddressErr>;
}

impl IntoOsc for i32 {
    type AsOsc = (Integer,);
    #[inline(always)]
    fn into_osc<Path: IntoAddress<Method>, Method: IntoIntoAddress>(
        self,
        path: Path,
        method: Method,
    ) -> Result<Message<Path, Method, Self::AsOsc>, AddressErr> {
        Ok(Message::new(
            path.into_address(method)?,
            (self
                .into_atomic()
                .map_err(|e| AddressErr::StringErr(e.into()))?,),
        ))
    }
}

impl IntoOsc for f32 {
    type AsOsc = (Float,);
    #[inline(always)]
    fn into_osc<Path: IntoAddress<Method>, Method: IntoIntoAddress>(
        self,
        path: Path,
        method: Method,
    ) -> Result<Message<Path, Method, Self::AsOsc>, AddressErr> {
        Ok(Message::new(
            path.into_address(method)?,
            (self
                .into_atomic()
                .map_err(|e| AddressErr::StringErr(e.into()))?,),
        ))
    }
}

impl<'s> IntoOsc for &'s str {
    type AsOsc = (String<'s>,);
    #[inline(always)]
    fn into_osc<Path: IntoAddress<Method>, Method: IntoIntoAddress>(
        self,
        path: Path,
        method: Method,
    ) -> Result<Message<Path, Method, Self::AsOsc>, AddressErr> {
        Ok(Message::new(
            path.into_address(method)?,
            (self.into_atomic().map_err(AddressErr::StringErr)?,),
        ))
    }
}

impl<'b> IntoOsc for &'b [u8] {
    type AsOsc = (Blob<'b>,);
    #[inline(always)]
    fn into_osc<Path: IntoAddress<Method>, Method: IntoIntoAddress>(
        self,
        path: Path,
        method: Method,
    ) -> Result<Message<Path, Method, Self::AsOsc>, AddressErr> {
        Ok(Message::new(
            path.into_address(method)?,
            (self
                .into_atomic()
                .map_err(|e| AddressErr::StringErr(e.into()))?,),
        ))
    }
}

impl IntoOsc for () {
    type AsOsc = ();
    #[inline(always)]
    fn into_osc<Path: IntoAddress<Method>, Method: IntoIntoAddress>(
        self,
        path: Path,
        method: Method,
    ) -> Result<Message<Path, Method, Self::AsOsc>, AddressErr> {
        Ok(Message::new(path.into_address(method)?, ()))
    }
}

/// Implement `IntoOsc` for a tuple of types, each of which implement `IntoAtomic`.
macro_rules! impl_for_tuple {
    ($($id:ident),+) => {
        impl<$($id: IntoAtomic),+> IntoOsc for ($($id),+,)
        where
            $(InvalidContents: From<<$id::AsAtomic as TryFrom<$id>>::Error>),+,
        {
            type AsOsc = ($($id::AsAtomic),+,);
            #[inline(always)]
            #[allow(non_snake_case)]
            fn into_osc<Path: IntoAddress<Method>, Method: IntoIntoAddress>(
                self,
                path: Path,
                method: Method,
            ) -> Result<Message<Path, Method, Self::AsOsc>, AddressErr> {
                let ($($id),+,) = self;
                Ok(Message::new(path.into_address(method)?, ($($id.into_atomic().map_err(|e| AddressErr::StringErr(e.into()))?),+,)))
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
impl IntoOsc for Data {
    type AsOsc = (Data,);
    #[inline(always)]
    fn into_osc<Path: IntoAddress<Method>, Method: IntoIntoAddress>(
        self,
        path: Path,
        method: Method,
    ) -> Result<Message<Path, Method, Self::AsOsc>, AddressErr> {
        Ok(Message::new(path.into_address(method)?, (self,)))
    }
}

#[cfg(feature = "alloc")]
impl IntoOsc for Dynamic {
    type AsOsc = Dynamic;
    #[inline(always)]
    fn into_osc<Path: IntoAddress<Method>, Method: IntoIntoAddress>(
        self,
        path: Path,
        method: Method,
    ) -> Result<Message<Path, Method, Self::AsOsc>, AddressErr> {
        Ok(Message::new(path.into_address(method)?, self))
    }
}

#[cfg(feature = "alloc")]
impl IntoOsc for alloc::string::String {
    type AsOsc = (DynamicString,);
    #[inline(always)]
    fn into_osc<Path: IntoAddress<Method>, Method: IntoIntoAddress>(
        self,
        path: Path,
        method: Method,
    ) -> Result<Message<Path, Method, Self::AsOsc>, AddressErr> {
        Ok(Message::new(
            path.into_address(method)?,
            (self.into_atomic().map_err(AddressErr::StringErr)?,),
        ))
    }
}

#[cfg(feature = "alloc")]
#[allow(unused_qualifications)]
impl IntoOsc for alloc::vec::Vec<u8> {
    type AsOsc = (DynamicBlob,);
    #[inline(always)]
    fn into_osc<Path: IntoAddress<Method>, Method: IntoIntoAddress>(
        self,
        path: Path,
        method: Method,
    ) -> Result<Message<Path, Method, Self::AsOsc>, AddressErr> {
        Ok(Message::new(
            path.into_address(method)?,
            (self
                .into_atomic()
                .map_err(|e| AddressErr::StringErr(e.into()))?,),
        ))
    }
}
