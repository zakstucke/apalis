use std::{
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use futures::{
    channel::mpsc::{channel, Receiver, Sender},
    Stream, StreamExt,
};

/// The `Notify` struct encapsulates asynchronous, multi-producer, single-consumer (MPSC) channel functionality.
/// It is used to send notifications of type `T` from multiple producers to a single consumer.
#[derive(Debug)]

pub struct Notify<T> {
    sender: Sender<T>,
    receiver: Arc<futures::lock::Mutex<Receiver<T>>>,
}

impl<T> Clone for Notify<T> {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
            receiver: self.receiver.clone(),
        }
    }
}

impl<T> Notify<T> {
    /// Creates a new instance of `Notify`.
    /// It initializes a channel with a buffer size of 10 and wraps the receiver in an `Arc<Mutex>`.
    pub fn new() -> Self {
        let (sender, receiver) = channel(10);

        Self {
            sender,
            receiver: Arc::new(futures::lock::Mutex::new(receiver)),
        }
    }

    /// Sends a notification of type `T` to the receiver.
    /// If the channel is full, the notification is silently dropped.
    pub fn notify(&self, value: T) {
        let _ = self.sender.clone().try_send(value);
    }

    /// Waits for and retrieves the next notification.
    /// This is an asynchronous method that awaits until a notification is available.
    /// Panics if the sender is dropped, ensuring that `notified` is always eventually fulfilled.
    pub async fn notified(&self) {
        self.receiver
            .lock()
            .await
            .next()
            .await
            .expect("sender is dropped");
    }
}

impl<T> Default for Notify<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Stream for Notify<T> {
    type Item = T;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if let Some(mut receiver) = self.receiver.try_lock() {
            receiver.poll_next_unpin(cx)
        } else {
            Poll::Pending
        }
    }
}

pub trait Notifier<T> {
    fn notify(&self, msg: T) -> Result<(), ()>;
}

impl<T> Notifier<T> for Notify<T> {
    fn notify(&self, msg: T) -> Result<(), ()> {
        self.notify(msg);
        Ok(())
    }
}
