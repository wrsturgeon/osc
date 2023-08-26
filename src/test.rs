/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

#![allow(
    clippy::default_numeric_fallback,
    clippy::print_stdout,
    clippy::unwrap_used,
    clippy::use_debug
)]

use crate::{IntoAtomic, IntoOsc, Tuple};

/// Examples from <https://opensoundcontrol.stanford.edu/spec-1_0-examples.html>.
mod from_the_spec {
    use crate::AddressErr;

    use super::*;

    #[test]
    fn string_osc() {
        assert!("osc".into_atomic().unwrap().into_iter().eq("osc\0".bytes()));
    }

    #[test]
    fn string_data() {
        assert!("data"
            .into_atomic()
            .unwrap()
            .into_iter()
            .eq("data\0\0\0\0".bytes()));
    }

    #[test]
    fn type_tag_f() {
        assert!(().type_tag().eq(core::iter::empty()));
    }

    #[test]
    #[allow(clippy::as_conversions)]
    fn type_tag_iisfff() {
        assert!((
            0.into_atomic().unwrap(),
            0.into_atomic().unwrap(),
            "".into_atomic().unwrap(),
            0.0.into_atomic().unwrap(),
            0.0.into_atomic().unwrap(),
            0.0.into_atomic().unwrap(),
        )
            .type_tag()
            .map(|tag| tag as u8)
            .eq("iisfff".bytes()));
    }

    #[test]
    #[allow(clippy::as_conversions)]
    fn type_tag_none() {
        assert!((0.0.into_atomic().unwrap(),)
            .type_tag()
            .map(|tag| tag as u8)
            .eq("f".bytes()));
    }

    #[test]
    #[allow(clippy::as_conversions)]
    fn type_tag_ibb() {
        assert!((
            0.into_atomic().unwrap(),
            (&[]).into_atomic().unwrap(),
            (&[]).into_atomic().unwrap()
        )
            .type_tag()
            .map(|tag| tag as u8)
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
        crate::{Address, Aligned4B, Decode, DynamicString, Message, Tag, Tags},
        quickcheck::quickcheck,
    };
    quickcheck! {
        #[allow(unused_variables)]
        fn message_doesnt_panic(message: Message<Vec<String>>) -> bool { true }

        fn four_byte_decode(v: Vec<u8>) -> bool {
            let size = v.len();
            let mut iter = v.into_iter();
            for _ in 0..(size >> 2) {
                if Aligned4B::<core::convert::Infallible>::decode(&mut iter).is_err() {
                    return false;
                }
            }
            if (size % 4) == 0 {
                iter.next().is_none()
            } else {
                Aligned4B::<core::convert::Infallible>::decode(&mut iter).is_err()
            }
        }

        fn string_roundtrip(original: DynamicString) -> bool {
            let decoded = DynamicString::decode(&mut original.clone().into_iter());
            // println!("{original:#?} --> {decoded:#?}");
            decoded == Ok(original)
        }

        #[allow(clippy::needless_collect)]
        fn string_roundtrip_bytes(original: Vec<u8>) -> bool {
            for _ in 0..(1 << 16) {
                let Ok(decoded) = DynamicString::decode(&mut original.iter().copied()) else { continue; };
                let recoded: Vec<_> = decoded.into_iter().collect();
                // println!("{original:#?} --> {recoded:#?}");
                for (a, b) in recoded.into_iter().zip(original.iter().copied()) { if a != b { return false; } }
            }
            true
        }

        fn address_roundtrip(original: Address<Vec<String>, String>) -> bool {
            let decoded = Address::decode(&mut original.clone().into_iter());
            // println!("{original:#?} --> {decoded:#?}");
            decoded == Ok(original)
        }

        #[allow(clippy::needless_collect)]
        fn address_roundtrip_bytes(original: Vec<u8>) -> bool {
            for _ in 0..(1 << 16) {
                let Ok(decoded) = Address::decode(&mut original.iter().copied()) else { continue; };
                let recoded: Vec<_> = decoded.into_iter().collect();
                // println!("{original:#?} --> {recoded:#?}");
                for (a, b) in recoded.into_iter().zip(original.iter().copied()) { if a != b { return false; } }
            }
            true
        }

        #[allow(clippy::as_conversions)]
        fn tag_byte_roundtrip(tag: Tag) -> bool {
            (tag as u8).try_into() == Ok(tag)
        }

        #[allow(clippy::as_conversions)]
        fn byte_tag_roundtrip(byte: u8) -> quickcheck::TestResult {
            Tag::try_from(byte).map_or_else(|_| quickcheck::TestResult::passed(), |tag| quickcheck::TestResult::from_bool((tag as u8) == byte))
        }

        fn tags_byte_roundtrip(original: Tags) -> bool {
            let mut encoded = original.clone().into_iter();
            let decoded = Tags::decode(&mut encoded);
            // println!("{original:#?} --> {decoded:#?}");
            decoded == Ok(original)
        }

        // fn data_roundtrip(original: Data) -> bool {
        //     let decoded = Data::decode(&mut original.clone().into_iter());
        //     println!("{original:#?} --> {decoded:#?}");
        //     decoded == Ok(original)
        // }
    }
}

mod prop_reduced {
    #[cfg(feature = "alloc")]
    use crate::{Address, Decode};

    #[test]
    #[cfg(feature = "alloc")]
    #[allow(clippy::similar_names)]
    fn address_roundtrip_bytes_reduced_1() {
        let original = alloc::vec![47, 1, 0, 1];
        let Ok(decoded) = Address::decode(&mut original.iter().copied()) else { return; };
        let recoded: Vec<_> = decoded.into_iter().collect();
        println!("{original:#?} --> {recoded:#?}");
        assert_eq!(recoded, original);
    }

    #[test]
    #[cfg(feature = "alloc")]
    #[allow(clippy::similar_names)]
    fn address_roundtrip_bytes_reduced_2() {
        let original = alloc::vec![47, 128, 0, 0];
        let Ok(decoded) = Address::decode(&mut original.iter().copied()) else { return; };
        let recoded: Vec<_> = decoded.into_iter().collect();
        println!("{original:#?} --> {recoded:#?}");
        assert_eq!(recoded, original);
    }
}

mod unit {
    #[cfg(feature = "alloc")]
    use crate::{Decode, Tag, Tags};

    #[test]
    #[cfg(feature = "alloc")]
    fn manual_tags_roundtrip() {
        for original in [
            Tags(vec![]),
            Tags(vec![Tag::Integer]),
            Tags(vec![Tag::Float]),
            Tags(vec![Tag::String]),
            Tags(vec![Tag::Blob]),
            Tags(vec![Tag::Integer, Tag::Integer]),
            Tags(vec![Tag::Integer, Tag::Float]),
            Tags(vec![Tag::Integer, Tag::String]),
            Tags(vec![Tag::Integer, Tag::Blob]),
            Tags(vec![Tag::Float, Tag::Integer]),
            Tags(vec![Tag::Float, Tag::Float]),
            Tags(vec![Tag::Float, Tag::String]),
            Tags(vec![Tag::Float, Tag::Blob]),
            Tags(vec![Tag::String, Tag::Integer]),
            Tags(vec![Tag::String, Tag::Float]),
            Tags(vec![Tag::String, Tag::String]),
            Tags(vec![Tag::String, Tag::Blob]),
            Tags(vec![Tag::Blob, Tag::Integer]),
            Tags(vec![Tag::Blob, Tag::Float]),
            Tags(vec![Tag::Blob, Tag::String]),
            Tags(vec![Tag::Blob, Tag::Blob]),
            Tags(vec![Tag::Integer, Tag::Integer, Tag::Integer]),
            Tags(vec![Tag::Integer, Tag::Integer, Tag::Integer, Tag::Integer]),
            Tags(vec![
                Tag::Integer,
                Tag::Integer,
                Tag::Integer,
                Tag::Integer,
                Tag::Integer,
            ]),
        ] {
            assert_eq!(
                Tags::decode(&mut original.clone().into_iter()),
                Ok(original)
            );
        }
    }
}
