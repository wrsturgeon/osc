/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Single-character OSC type tags.

/// Single-character OSC type tag.
#[repr(u8)]
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Tag {
    /// 32-bit big-endian signed two's-complement integer.
    Integer = b'i',
    /// 32-bit big-endian IEEE 754 floating-point number.
    Float = b'f',
    /// Null-terminated (not your responsibility!) byte string.
    String = b's',
    /// Arbitrary known-length collection of bytes.
    Blob = b'b',
}
