

use tokio::sync::oneshot;
use tokio::time::{Duration, Sleep};
use std::pin::Pin;
use std::future::Future;
use std::task::{Context, Poll};

use crate::{ActorError, Channel};

pub struct DelayedMessage<T> {
    pub value: Option<T>,
    pub delay: Pin<Box<Sleep>>,
}

impl<T> DelayedMessage<T> {
    pub fn new(value: T, delay: Duration) -> Self {
        DelayedMessage {
            value: Some(value),
            delay: Box::pin(tokio::time::sleep(delay)),
        }
    }
}

impl<T: Send> Channel<T> for DelayedMessage<T> {
    fn recv(&mut self) -> Pin<Box<dyn Future<Output = Result<T, ActorError>> + Send + '_>> {
        Box::pin(DelayedMessageFuture {
            value: &mut self.value,
            delay: self.delay.as_mut(),
        })
    }
}

struct DelayedMessageFuture<'a, T> {
    value: &'a mut Option<T>,
    delay: Pin<&'a mut Sleep>,
}

impl<'a, T> Future for DelayedMessageFuture<'a, T> {
    type Output = Result<T, ActorError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // First, poll the delay
        match self.delay.as_mut().poll(cx) {
            Poll::Ready(_) => {
                // Delay is over, try to take the value
                match self.value.take() {
                    Some(value) => Poll::Ready(Ok(value)),
                    None => Poll::Ready(Err(ActorError::ActorClosed)),
                }
            }
            Poll::Pending => Poll::Pending,
        }
    }
}