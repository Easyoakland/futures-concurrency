use super::Merge as MergeTrait;
use crate::stream::IntoStream;
use crate::utils;

use core::fmt;
use futures_core::Stream;
use std::pin::Pin;
use std::task::{Context, Poll};

// TODO: handle none case
macro_rules! impl_merge_tuple {
    ($StructName:ident) => {
        /// A stream that merges multiple streams into a single stream.
        ///
        /// This `struct` is created by the [`merge`] method on the [`Merge`] trait. See its
        /// documentation for more.
        ///
        /// [`merge`]: trait.Merge.html#method.merge
        /// [`Merge`]: trait.Merge.html
        #[pin_project::pin_project]
        pub struct $StructName {
            rng: utils::RandomGenerator,
        }

        impl fmt::Debug for $StructName {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_tuple("Merge").finish()
            }
        }

        impl Stream for $StructName {
            type Item = std::convert::Infallible; // TODO: convert to `never` type in the stdlib

            fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
                Poll::Ready(None)
            }
        }

        impl MergeTrait for () {
            type Item = std::convert::Infallible; // TODO: convert to `never` type in the stdlib
            type Stream = $StructName;

            fn merge(self) -> Self::Stream {
                $StructName {
                    rng: utils::RandomGenerator::new(),
                }
            }
        }
    };
    ($StructName:ident $($F:ident)+) => {
        /// A stream that merges multiple streams into a single stream.
        ///
        /// This `struct` is created by the [`merge`] method on the [`Merge`] trait. See its
        /// documentation for more.
        ///
        /// [`merge`]: trait.Merge.html#method.merge
        /// [`Merge`]: trait.Merge.html
        #[pin_project::pin_project]
        pub struct $StructName<T, $($F),*>
        where $(
            $F: Stream<Item = T>,
        )* {
            done: bool,
            $(#[pin] $F: $F,)*
            rng: utils::RandomGenerator,
        }

        impl<T, $($F),*> fmt::Debug for $StructName<T, $($F),*>
        where $(
            $F: Stream<Item = T> + fmt::Debug,
            T: fmt::Debug,
        )* {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_tuple("Merge")
                    $(.field(&self.$F))*
                    .finish()
            }
        }

        impl<T, $($F),*> Stream for $StructName<T, $($F),*>
        where $(
            $F: Stream<Item = T>,
        )* {
            type Item = T;

            fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
                let mut this = self.project();

                // Return early in case we're polled again after completion.
                if *this.done {
                    return Poll::Ready(None);
                }

                const LEN: u32 = utils::tuple_len!($($F,)*);
                const PERMUTATIONS: u32 = utils::permutations(LEN);
                let r = this.rng.generate(PERMUTATIONS);
                let mut pending = false;
                for i in 0..LEN {
                    utils::gen_conditions!(LEN, i, r, this, cx, poll_next, {
                        Poll::Ready(Some(value)) => return Poll::Ready(Some(value)),
                        Poll::Ready(None) => continue,
                        Poll::Pending => {
                            pending = true;
                            continue
                        },
                    }, $($F,)*);
                }
                if pending {
                    Poll::Pending
                } else {
                    *this.done = true;
                    Poll::Ready(None)
                }
            }
        }

        impl<T, $($F),*> MergeTrait for ($($F,)*)
        where $(
            $F: IntoStream<Item = T>,
        )* {
            type Item = T;
            type Stream = $StructName<T, $($F::IntoStream),*>;

            fn merge(self) -> Self::Stream {
                let ($($F,)*): ($($F,)*) = self;
                $StructName {
                    done: false,
                    rng: utils::RandomGenerator::new(),
                    $($F: $F.into_stream()),*
                }
            }
        }
    };
}

impl_merge_tuple! { Merge0  }
impl_merge_tuple! { Merge1  A }
impl_merge_tuple! { Merge2  A B }
impl_merge_tuple! { Merge3  A B C }
impl_merge_tuple! { Merge4  A B C D }
impl_merge_tuple! { Merge5  A B C D E }
impl_merge_tuple! { Merge6  A B C D E F }
impl_merge_tuple! { Merge7  A B C D E F G }
impl_merge_tuple! { Merge8  A B C D E F G H }
impl_merge_tuple! { Merge9  A B C D E F G H I }
impl_merge_tuple! { Merge10 A B C D E F G H I J }
impl_merge_tuple! { Merge11 A B C D E F G H I J K }
impl_merge_tuple! { Merge12 A B C D E F G H I J K L }

#[cfg(test)]
mod tests {
    use super::*;
    use futures_lite::future::block_on;
    use futures_lite::prelude::*;
    use futures_lite::stream;

    #[test]
    fn merge_tuple_0() {
        block_on(async {
            let mut s = ().merge();

            let mut called = false;
            while let Some(_) = s.next().await {
                called = true;
            }
            assert!(!called);
        })
    }

    #[test]
    fn merge_tuple_1() {
        block_on(async {
            let a = stream::once(1);
            let mut s = (a,).merge();

            let mut counter = 0;
            while let Some(n) = s.next().await {
                counter += n;
            }
            assert_eq!(counter, 1);
        })
    }

    #[test]
    fn merge_tuple_2() {
        block_on(async {
            let a = stream::once(1);
            let b = stream::once(2);
            let mut s = (a, b).merge();

            let mut counter = 0;
            while let Some(n) = s.next().await {
                counter += n;
            }
            assert_eq!(counter, 3);
        })
    }

    #[test]
    fn merge_tuple_3() {
        block_on(async {
            let a = stream::once(1);
            let b = stream::once(2);
            let c = stream::once(3);
            let mut s = (a, b, c).merge();

            let mut counter = 0;
            while let Some(n) = s.next().await {
                counter += n;
            }
            assert_eq!(counter, 6);
        })
    }

    #[test]
    fn merge_tuple_4() {
        block_on(async {
            let a = stream::once(1);
            let b = stream::once(2);
            let c = stream::once(3);
            let d = stream::once(4);
            let mut s = (a, b, c, d).merge();

            let mut counter = 0;
            while let Some(n) = s.next().await {
                counter += n;
            }
            assert_eq!(counter, 10);
        })
    }
}
