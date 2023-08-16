/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

#![allow(clippy::default_numeric_fallback, clippy::unwrap_used)]

use crate::{IntoAtomic, IntoOsc, Tuple};

/// Examples from <https://opensoundcontrol.stanford.edu/spec-1_0-examples.html>.
mod from_the_spec {
    use crate::AddressErr;

    use super::*;

    #[test]
    fn string_osc() {
        assert!("osc".into_atomic().into_iter().eq("osc\0".bytes()));
    }

    #[test]
    fn string_data() {
        assert!("data".into_atomic().into_iter().eq("data\0\0\0\0".bytes()));
    }

    #[test]
    fn type_tag_f() {
        assert!(().type_tag().eq(core::iter::empty()));
    }

    #[test]
    fn type_tag_iisfff() {
        assert!((
            0.into_atomic(),
            0.into_atomic(),
            "".into_atomic(),
            0.0.into_atomic(),
            0.0.into_atomic(),
            0.0.into_atomic(),
        )
            .type_tag()
            .eq("iisfff".bytes()));
    }

    #[test]
    fn type_tag_none() {
        assert!((0.0.into_atomic(),).type_tag().eq("f".bytes()));
    }

    #[test]
    fn type_tag_ibb() {
        assert!((0.into_atomic(), (&[]).into_atomic(), (&[]).into_atomic())
            .type_tag()
            .eq("ibb".bytes()));
    }

    #[test]
    fn message_oscillator_4_frequency() {
        let msg = (440.).into_osc(["oscillator", "4"], "frequency").unwrap();
        assert!(msg.into_iter().eq(b"\
            /oscillator/4/frequency\0\
            ,f\0\0\
            \x43\xDC\x00\x00"
            .iter()
            .copied()));
    }

    #[test]
    #[allow(clippy::panic_in_result_fn)]
    fn message_foo() -> Result<(), AddressErr> {
        let osc = (1000, -1, "hello", 1.234, 5.678).into_osc([], "foo")?;
        let by_hand = b"\
            /foo\0\0\0\0\
            ,iisff\0\0\
            \x00\x00\x03\xE8\
            \xFF\xFF\xFF\xFF\
            hello\0\0\0\
            \x3F\x9D\xF3\xB6\
            \x40\xB5\xB2\x2D";
        assert!(osc.into_iter().eq(by_hand.iter().copied()));
        Ok(())
    }
}

#[cfg(feature = "quickcheck")]
mod prop {
    use {
        crate::{Address, Aligned4B, Decode, Message},
        quickcheck::quickcheck,
    };
    quickcheck! {

        #[allow(unused_variables)]
        fn message_doesnt_panic(message: Message<Vec<String>>) -> bool { true }

        fn four_byte_decode(v: Vec<u8>) -> bool {
            let size = v.len();
            let mut iter = v.into_iter();
            for _ in 0..(size >> 2) {
                if Aligned4B::decode(&mut iter).is_err() {
                    return false;
                }
            }
            if (size % 4) == 0 {
                iter.next().is_none()
            } else {
                Aligned4B::decode(&mut iter).is_err()
            }
        }

        fn address_roundtrip(address: Address<Vec<String>, String>) -> bool {
            let redecoded = Address::decode(&mut address.clone().into_iter());
            println!("{address:#?} --> {redecoded:#?}");
            redecoded == Ok(address)
        }

    }
}
