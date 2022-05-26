#[macro_use]
extern crate macro_rules_attribute;

use ::nougat::*;

#[gat]
trait LendingIterator<> {
    type Item<'next>
    where
        Self : 'next,
    ;

    fn next (
        self: &'_ mut Self,
    ) -> Option<Self::Item<'_>>
    ;
}

struct Infinite;

#[gat]
impl LendingIterator for Infinite {
    type Item<'next>
    where
        Self : 'next,
    =
        &'next mut Self
    ;

    fn next (
        self: &'_ mut Self,
    ) -> Option<&'_ mut Self>
    {
        Some(self)
    }
}

fn check<I : LendingIterator> (mut iter: I)
{
    let _ = check::<Infinite>;
    while let Some(_item) = iter.next() {
        // …
    }
}

// trait LendingIterator2__Item<'next, SelfLt = Self, Bounds = &'next SelfLt> {
//     type T : ?Sized + Sized;
// }

// trait LendingIterator2<SelfLt> {
//     type Item :
//         ?Sized + for<'next> LendingIterator2__Item<'next, SelfLt>
//     ;

//     fn next (self: &'_ mut Self)
//       -> <
//             <Self as LendingIterator2<SelfLt>>::Item
//             as
//             LendingIterator2__Item<'_, SelfLt>
//         >::T
//     ;
// }

// type Foo<'outlives> = dyn 'outlives + LendingIterator2<
//     &'outlives (),
//     Item = dyn for<'next> LendingIterator2__Item<'next, &'outlives (), T = &'next u8>,
// >;

// // #[adjugate]
// trait LendingIterator<SelfLt = Self>
// :
//     for<'next> LendingIterator__Item<'next, SelfLt, &'next SelfLt>
// {
//     fn next<'next> (
//         self: &'next mut Self,
//     ) -> Option< Gat!(<Self as LendingIterator<SelfLt>>::Item<'next>) >
//     ;
// }
// trait LendingIterator__Item<'next, SelfLt = Self, Bound = &'next SelfLt> {
//     type T;
// }

// #[adjugate]
// type Bar<'__, T> = <T as LendingIterator>::Item<'__>;
// type Baz<'__, T> = Gat!(<T as LendingIterator>::Item<'__>);
// #[apply(Gat!)]
// type Quux<'__, T> = <T as LendingIterator>::Item<'__>;

// Self::Item<'next>
// <Self as LendingIterator>::Item<'next>
