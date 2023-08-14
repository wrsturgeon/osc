/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub use implementation::*;

#[cfg(not(any(test, feature = "quickcheck")))]
mod implementation {
    pub trait QuickCheckIfEnabled {}
    pub trait CloneIfQuickCheck {}

    impl<A /* unconditionally */> QuickCheckIfEnabled for A {}
    impl<A /* unconditionally */> CloneIfQuickCheck for A {}
}

#[cfg(any(test, feature = "quickcheck"))]
mod implementation {
    use crate::{Blob, Float, Integer, IntoOsc, String, TimeTag};
    use quickcheck::Arbitrary;

    pub trait QuickCheckIfEnabled: Arbitrary + core::fmt::Debug {}
    pub trait CloneIfQuickCheck: 'static + Clone + PartialEq + core::fmt::Debug {}

    impl<A: Arbitrary + core::fmt::Debug> QuickCheckIfEnabled for A {}
    impl<A: 'static + Clone + PartialEq + core::fmt::Debug> CloneIfQuickCheck for A {}

    impl Arbitrary for Integer {
        #[inline(always)]
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            i32::arbitrary(g).into_osc()
        }
        #[inline(always)]
        #[allow(unused_qualifications)]
        fn shrink(&self) -> alloc::boxed::Box<dyn Iterator<Item = Self>> {
            i32::from_be_bytes(self.0).shrink()
        }
    }

    impl Arbitrary for Float {
        #[inline(always)]
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            f32::arbitrary(g).into_osc()
        }
        #[inline(always)]
        #[allow(unused_qualifications)]
        fn shrink(&self) -> alloc::boxed::Box<dyn Iterator<Item = Self>> {
            f32::from_be_bytes(self.0).shrink()
        }
    }

    impl Arbitrary for TimeTag {
        #[inline(always)]
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            TimeTag {
                seconds: i32::arbitrary(g).to_be_bytes(),
                sub_second: i32::arbitrary(g).to_be_bytes(),
            }
        }
        #[inline(always)]
        #[allow(unused_qualifications)]
        fn shrink(&self) -> alloc::boxed::Box<dyn Iterator<Item = Self>> {
            ((i64::from(self.seconds) << 32) | self.sub_second).shrink()
        }
    }

    impl<S: 'static + Clone + Iterator<Item = u8>> Arbitrary for String<S> {
        #[inline(always)]
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            alloc::string::String::arbitrary(g).into_osc()
        }
        #[inline(always)]
        fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
            self.0.unbatch().collect::<Vec<_>>().shrink()
        }
    }

    impl<B: 'static + Clone + ExactSizeIterator<Item = u8>> Arbitrary for Blob<B> {
        #[inline(always)]
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            alloc::vec::Vec::arbitrary(g).into_osc()
        }
        #[inline(always)]
        fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
            self.0.unbatch().collect::<Vec<_>>().shrink()
        }
    }
}
