use crate::stream::Stream;

use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use pin_project_lite::pin_project;

// Do not export this struct until `FromStream` can be unsealed.
pin_project! {
    /// Stream returned by the [`collect`](super::StreamExt::collect) method.
    #[must_use = "streams do nothing unless polled"]
    #[derive(Debug)]
    pub struct Collect<T, U>
    where
        T: Stream,
        U: FromStream<T::Item>,
    {
        #[pin]
        stream: T,
        collection: U::Collection,
    }
}

/// Convert from a [`Stream`](crate::stream::Stream).
///
/// This trait is not intended to be used directly. Instead, call
/// [`StreamExt::collect()`](super::StreamExt::collect).
///
/// # Implementing
///
/// Currently, this trait may not be implemented by third parties. The trait is
/// sealed in order to make changes in the future. Stabilization is pending
/// enhancements to the Rust langague.
pub trait FromStream<T>: sealed::FromStreamPriv<T> {}

impl<T, U> Collect<T, U>
where
    T: Stream,
    U: FromStream<T::Item>,
{
    pub(super) fn new(stream: T) -> Collect<T, U> {
        let (lower, upper) = stream.size_hint();
        let collection = U::initialize(lower, upper);

        Collect {
            stream,
            collection,
        }
    }
}

impl<T, U> Future for Collect<T, U>
where
    T: Stream,
    U: FromStream<T::Item>,
{
    type Output = U;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<U> {
        use Poll::Ready;

        loop {
            let mut me = self.as_mut().project();

            let item = match ready!(me.stream.poll_next(cx)) {
                Some(item) => item,
                None => {
                    return Ready(U::finalize(&mut me.collection));
                }
            };

            U::extend(&mut me.collection, item);
        }
    }
}

pub(crate) mod sealed {
    #[doc(hidden)]
    pub trait FromStreamPriv<T> {
        /// Intermediate type used during collection process
        type Collection;

        /// Initialize the collection
        fn initialize(lower: usize, upper: Option<usize>) -> Self::Collection;

        /// Extend the collection with the received item
        fn extend(collection: &mut Self::Collection, item: T);

        /// Finalize collection into target type.
        fn finalize(collection: &mut Self::Collection) -> Self;
    }

    impl FromStreamPriv
}
