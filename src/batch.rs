/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Align an iterator to 4-byte batches by padding with zeros at the end.

use core::mem::MaybeUninit;

/// Three-byte buffer.
#[repr(packed)]
#[derive(Clone, Copy, Debug)]
struct Cache {
    /// Three-byte buffer.
    buffer: MaybeUninit<[u8; 3]>,
    /// Index from 0 to 3.
    index: u8,
}

impl Default for Cache {
    #[inline(always)]
    #[allow(unsafe_code)]
    fn default() -> Self {
        Self {
            buffer: MaybeUninit::uninit(),
            index: 3,
        }
    }
}

impl Cache {
    /// Initialize a cache by pulling four bytes, caching the last three and returning the first.
    fn new<I: Iterator<Item = u8>>(iter: &mut I) -> Self {
        Self {
            buffer: MaybeUninit::new([
                iter.next().unwrap_or(0),
                iter.next().unwrap_or(0),
                iter.next().unwrap_or(0),
            ]),
            index: 0,
        }
    }
}

#[allow(clippy::copy_iterator)]
impl Iterator for Cache {
    type Item = u8;
    #[inline]
    #[allow(clippy::arithmetic_side_effects, unsafe_code)]
    fn next(&mut self) -> Option<Self::Item> {
        (self.index < 3).then(|| {
            let i = usize::from(self.index);
            self.index += 1;
            // SAFETY:
            // Just checked above. If `3` ever changes, revisit.
            unsafe { *self.buffer.assume_init().get_unchecked(i) }
        })
    }
}

/// Align an iterator to 4-byte batches by padding with zeros at the end.
#[derive(Clone, Copy, Debug, Default)]
pub struct Batched<I: Iterator<Item = u8>> {
    /// Iterator over individual bytes.
    iter: I,
    /// 4-byte cache.
    cache: Cache,
}

impl<I: Iterator<Item = u8>> Batched<I> {
    /// Batch an iterator into four-byte chunks, padding the end with zeros.
    /// Note that this is a lazy operation.
    #[inline]
    pub fn new(iter: I) -> Self {
        Self {
            iter,
            cache: Cache::default(),
        }
    }
    /// Un-batch into the original iterator
    #[inline]
    #[allow(clippy::missing_const_for_fn)]
    pub fn unbatch(self) -> I {
        self.iter
    }
}

impl<I: Iterator<Item = u8>> Iterator for Batched<I> {
    type Item = u8;
    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        self.cache.next().or_else(|| {
            let tmp = self.iter.next();
            if tmp.is_some() {
                self.cache = Cache::new(&mut self.iter);
            }
            tmp
        })
    }
}

/// Call `into_iter` and lazily batch the iterator into four-byte chunks, padding the end with zeros.
pub trait Batch: IntoIterator<Item = u8> {
    /// Call `into_iter` and lazily batch the iterator into four-byte chunks, padding the end with zeros.
    fn batch(self) -> Batched<Self::IntoIter>;
}

impl<I: IntoIterator<Item = u8>> Batch for I {
    #[inline(always)]
    fn batch(self) -> Batched<Self::IntoIter> {
        Batched::new(self.into_iter())
    }
}
