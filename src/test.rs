/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::{Blob, Float, Integer, IntoAtomic, IntoOsc, String, Tuple};

/// Examples from <https://opensoundcontrol.stanford.edu/spec-1_0-examples.html>.
mod from_the_spec {
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
        assert!(<() as Tuple>::type_tag().eq(",\0\0\0".bytes()))
    }

    #[test]
    fn type_tag_iisfff() {
        assert!(
            <(Integer, Integer, String<'_>, Float, Float, Float) as Tuple>::type_tag()
                .eq(",iisfff\0".bytes())
        )
    }

    #[test]
    fn type_tag_none() {
        assert!(<(Float,) as Tuple>::type_tag().eq(",f\0\0".bytes()))
    }

    #[test]
    fn type_tag_ibb() {
        assert!(<(Integer, Blob<'_>, Blob<'_>) as Tuple>::type_tag().eq(",ibb".bytes()))
    }

    #[test]
    fn message_oscillator_4_frequency() {
        let msg = (440.).into_osc(["oscillator", "4", "frequency"]).unwrap();
        assert!(msg.eq(b"/oscillator/4/frequency\0,f\0\0\x43\xDC\0\0"
            .into_iter()
            .copied()))
    }

    #[test]
    fn message_foo() {
        let msg = (1000, -1, "hello", 1.234, 5.678).into_osc(["foo"]).unwrap();
        assert!(msg.eq(b"\
            /foo\0\0\0\0\
            ,iisff\0\0\
            \x00\x00\x03\xE8\
            \xFF\xFF\xFF\xFF\
            hello\0\0\0\
            \x3F\x9D\xF3\xB6\
            \x40\xB5\xB2\x2D"
            .iter()
            .copied()))
    }
}
