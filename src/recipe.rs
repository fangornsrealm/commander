//! Wrapping a pre-built stream as an iced [`Recipe`].

use cosmic::iced::advanced::subscription::{EventStream, Hasher, Recipe};
use cosmic::iced::futures::Stream;
use cosmic::iced::futures::stream::BoxStream;
use std::hash::Hash;

/// A [`Recipe`] wrapping a pre-built stream.
///
/// Needed for streams whose builder closure captures state and therefore
/// cannot be described by a stateless `Subscription::run` builder.
pub struct ReadyStream<S, I>(pub S, pub I)
where
    I: std::hash::Hash;

impl<S, I, T> Recipe for ReadyStream<S, I>
where
    S: Stream<Item = T> + Send + 'static,
    I: std::hash::Hash + 'static,
    T: Send + 'static,
{
    type Output = T;

    fn hash(&self, state: &mut Hasher) {
        std::any::type_name::<S>().hash(state);
        self.1.hash(state);
    }

    fn stream(self: Box<Self>, _input: EventStream) -> BoxStream<'static, Self::Output> {
        Box::pin(self.0)
    }
}
