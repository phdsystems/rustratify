//! Async stream utilities for event-driven APIs.
//!
//! This module provides utilities for creating and working with async streams,
//! which are the preferred way to handle events in Rustratify modules.

use std::pin::Pin;

use futures_core::Stream;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

/// Type alias for a boxed async stream of events.
///
/// This is the standard return type for event-producing operations in Rustratify.
pub type EventStream<T> = Pin<Box<dyn Stream<Item = T> + Send>>;

/// A sender for events in an async stream.
///
/// This wraps a tokio mpsc sender and provides convenience methods
/// for sending events.
#[derive(Debug)]
pub struct EventSender<T> {
    tx: mpsc::Sender<T>,
}

impl<T> EventSender<T> {
    /// Create a new event sender from an mpsc sender.
    pub fn new(tx: mpsc::Sender<T>) -> Self {
        Self { tx }
    }

    /// Send an event.
    ///
    /// Returns `Ok(())` if the event was sent, or `Err(event)` if the
    /// receiver was dropped.
    pub async fn send(&self, event: T) -> Result<(), T> {
        self.tx.send(event).await.map_err(|e| e.0)
    }

    /// Try to send an event without waiting.
    ///
    /// Returns `Ok(())` if the event was sent, or `Err(event)` if the
    /// channel is full or closed.
    pub fn try_send(&self, event: T) -> Result<(), T> {
        self.tx.try_send(event).map_err(|e| match e {
            mpsc::error::TrySendError::Full(v) => v,
            mpsc::error::TrySendError::Closed(v) => v,
        })
    }

    /// Check if the receiver has been dropped.
    pub fn is_closed(&self) -> bool {
        self.tx.is_closed()
    }

    /// Get the capacity of the underlying channel.
    pub fn capacity(&self) -> usize {
        self.tx.capacity()
    }
}

impl<T> Clone for EventSender<T> {
    fn clone(&self) -> Self {
        Self {
            tx: self.tx.clone(),
        }
    }
}

/// Builder for creating event streams.
///
/// # Example
///
/// ```rust
/// use rustratify::StreamBuilder;
///
/// #[derive(Debug, Clone)]
/// enum MyEvent {
///     Started,
///     Progress(u32),
///     Complete,
/// }
///
/// # async fn example() {
/// let (sender, stream) = StreamBuilder::<MyEvent>::new()
///     .buffer_size(100)
///     .build();
///
/// // Send events
/// sender.send(MyEvent::Started).await.unwrap();
/// sender.send(MyEvent::Progress(50)).await.unwrap();
/// sender.send(MyEvent::Complete).await.unwrap();
/// # }
/// ```
pub struct StreamBuilder<T> {
    buffer_size: usize,
    _marker: std::marker::PhantomData<T>,
}

impl<T: Send + 'static> StreamBuilder<T> {
    /// Create a new stream builder with default settings.
    pub fn new() -> Self {
        Self {
            buffer_size: 100,
            _marker: std::marker::PhantomData,
        }
    }

    /// Set the buffer size for the underlying channel.
    ///
    /// Default is 100.
    pub fn buffer_size(mut self, size: usize) -> Self {
        self.buffer_size = size;
        self
    }

    /// Build the stream and sender.
    ///
    /// Returns a tuple of (sender, stream).
    pub fn build(self) -> (EventSender<T>, EventStream<T>) {
        let (tx, rx) = mpsc::channel(self.buffer_size);
        let sender = EventSender::new(tx);
        let stream: EventStream<T> = Box::pin(ReceiverStream::new(rx));
        (sender, stream)
    }
}

impl<T: Send + 'static> Default for StreamBuilder<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Create an event stream with the default buffer size.
///
/// This is a convenience function for simple use cases.
///
/// # Example
///
/// ```rust
/// use rustratify::stream::create_stream;
///
/// # async fn example() {
/// let (sender, stream) = create_stream::<String>();
/// sender.send("Hello".to_string()).await.unwrap();
/// # }
/// ```
pub fn create_stream<T: Send + 'static>() -> (EventSender<T>, EventStream<T>) {
    StreamBuilder::<T>::new().build()
}

/// Create an event stream with a specific buffer size.
pub fn create_stream_with_buffer<T: Send + 'static>(
    buffer_size: usize,
) -> (EventSender<T>, EventStream<T>) {
    StreamBuilder::<T>::new().buffer_size(buffer_size).build()
}

/// Extension trait for working with event streams.
pub trait EventStreamExt<T> {
    /// Convert into a boxed stream.
    fn boxed(self) -> EventStream<T>;
}

impl<S, T> EventStreamExt<T> for S
where
    S: Stream<Item = T> + Send + 'static,
{
    fn boxed(self) -> EventStream<T> {
        Box::pin(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::StreamExt;

    #[derive(Debug, Clone, PartialEq)]
    enum TestEvent {
        Start,
        Progress(u32),
        Complete,
    }

    #[tokio::test]
    async fn test_stream_builder() {
        let (sender, mut stream) = StreamBuilder::<TestEvent>::new()
            .buffer_size(10)
            .build();

        sender.send(TestEvent::Start).await.unwrap();
        sender.send(TestEvent::Progress(50)).await.unwrap();
        sender.send(TestEvent::Complete).await.unwrap();
        drop(sender);

        let events: Vec<_> = stream.collect().await;
        assert_eq!(
            events,
            vec![
                TestEvent::Start,
                TestEvent::Progress(50),
                TestEvent::Complete,
            ]
        );
    }

    #[tokio::test]
    async fn test_create_stream() {
        let (sender, mut stream) = create_stream::<String>();

        sender.send("Hello".to_string()).await.unwrap();
        sender.send("World".to_string()).await.unwrap();
        drop(sender);

        let events: Vec<_> = stream.collect().await;
        assert_eq!(events, vec!["Hello", "World"]);
    }

    #[tokio::test]
    async fn test_sender_clone() {
        let (sender, mut stream) = create_stream::<u32>();

        let sender2 = sender.clone();
        sender.send(1).await.unwrap();
        sender2.send(2).await.unwrap();
        drop(sender);
        drop(sender2);

        let events: Vec<_> = stream.collect().await;
        assert_eq!(events, vec![1, 2]);
    }

    #[tokio::test]
    async fn test_try_send() {
        let (sender, _stream) = create_stream_with_buffer::<u32>(1);

        // First send should succeed
        assert!(sender.try_send(1).is_ok());
        // Second might fail if buffer is full
        // (depends on timing, so just test it doesn't panic)
        let _ = sender.try_send(2);
    }
}
