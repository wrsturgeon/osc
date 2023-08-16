/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Typed collection of data.

use crate::Atomic;
use core::iter::Chain;

/// Typed collection of data.
pub trait Tuple {
    /// Iterator over characters in the formatted type tag.
    type TypeTagIter: Iterator<Item = u8>;
    /// Format an OSC type tag for this collection of types.
    fn type_tag(&self) -> Self::TypeTagIter;
    /// Chained iterators over each piece of data in this tuple.
    type Chained: Iterator<Item = u8>;
    /// Chain iterators over each piece of data in this tuple.
    fn chain(self) -> Self::Chained;
}

impl Tuple for () {
    type TypeTagIter = core::iter::Empty<u8>;
    #[inline(always)]
    fn type_tag(&self) -> Self::TypeTagIter {
        core::iter::empty()
    }
    type Chained = core::iter::Empty<u8>;
    #[inline(always)]
    fn chain(self) -> Self::Chained {
        core::iter::empty()
    }
}

/// Implement `Tuple` for a tuple of types, each of which implement `Atomic`.
macro_rules! impl_tuple {
    ($n:expr, $chain:ty, |$self:ident| $chain_expr:expr, $($id:ident),+,) => {
        impl<$($id: Atomic),+> Tuple for ($($id),+,) {
            type TypeTagIter = core::array::IntoIter<u8, $n>;
            #[inline(always)]
            fn type_tag(&self) -> Self::TypeTagIter {
                #[allow(non_snake_case)]
                let &($(ref $id),+,) = self;
                [$($id.type_tag()),+].into_iter()
            }
            type Chained = $chain;
            #[inline]
            fn chain($self) -> Self::Chained {
                $chain_expr
            }
        }
    };
}

impl_tuple!(
    1,
    A::IntoIter,
    |self| self.0.into_iter(),
    A, //
);
impl_tuple!(
    2,
    Chain<A::IntoIter, B::IntoIter>,
    |self| self.0.into_iter().chain(self.1),
    A, B, //
);
impl_tuple!(
    3,
    Chain<Chain<A::IntoIter, B::IntoIter>, C::IntoIter>,
    |self| self.0.into_iter().chain(self.1).chain(self.2),
    A,
    B,
    C, //
);
impl_tuple!(
    4,
    Chain<Chain<Chain<A::IntoIter, B::IntoIter>, C::IntoIter>, D::IntoIter>,
    |self| self.0.into_iter().chain(self.1).chain(self.2).chain(self.3),
    A,
    B,
    C,
    D, //
);
impl_tuple!(
    5,
    Chain<Chain<Chain<Chain<A::IntoIter, B::IntoIter>, C::IntoIter>, D::IntoIter>, E::IntoIter>,
    |self| self
        .0
        .into_iter()
        .chain(self.1)
        .chain(self.2)
        .chain(self.3)
        .chain(self.4),
    A,
    B,
    C,
    D,
    E, //
);
impl_tuple!(
    6,
    Chain<
        Chain<Chain<Chain<Chain<A::IntoIter, B::IntoIter>, C::IntoIter>, D::IntoIter>, E::IntoIter>,
        F::IntoIter,
    >,
    |self| self
        .0
        .into_iter()
        .chain(self.1)
        .chain(self.2)
        .chain(self.3)
        .chain(self.4)
        .chain(self.5),
    A,
    B,
    C,
    D,
    E,
    F, //
);
impl_tuple!(
    7,
    Chain<
        Chain<
            Chain<
                Chain<Chain<Chain<A::IntoIter, B::IntoIter>, C::IntoIter>, D::IntoIter>,
                E::IntoIter,
            >,
            F::IntoIter,
        >,
        G::IntoIter,
    >,
    |self| self
        .0
        .into_iter()
        .chain(self.1)
        .chain(self.2)
        .chain(self.3)
        .chain(self.4)
        .chain(self.5)
        .chain(self.6),
    A,
    B,
    C,
    D,
    E,
    F,
    G, //
);
impl_tuple!(
    8,
    Chain<
        Chain<
            Chain<
                Chain<
                    Chain<Chain<Chain<A::IntoIter, B::IntoIter>, C::IntoIter>, D::IntoIter>,
                    E::IntoIter,
                >,
                F::IntoIter,
            >,
            G::IntoIter,
        >,
        H::IntoIter,
    >,
    |self| self
        .0
        .into_iter()
        .chain(self.1)
        .chain(self.2)
        .chain(self.3)
        .chain(self.4)
        .chain(self.5)
        .chain(self.6)
        .chain(self.7),
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H, //
);

#[cfg(feature = "alloc")]
#[allow(unused_qualifications)]
impl Tuple for alloc::vec::Vec<crate::Dynamic> {
    type TypeTagIter = alloc::vec::IntoIter<u8>;
    type Chained = core::iter::Flatten<alloc::vec::IntoIter<crate::Dynamic>>;
    #[inline]
    fn type_tag(&self) -> Self::TypeTagIter {
        self.iter()
            .map(crate::Dynamic::type_tag)
            .collect::<alloc::vec::Vec<_>>()
            .into_iter()
    }
    #[inline]
    #[allow(clippy::as_underscore, trivial_casts)]
    fn chain(self) -> Self::Chained {
        self.into_iter().flatten()
    }
}
