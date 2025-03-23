use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use crate::{ActorError, Channel};

pub struct Singleshot<T> {
    pub value: Option<T>,
}

impl<T> Singleshot<T> {
    pub fn new(value: T) -> Self {
        Singleshot { value: Some(value) }
    }
}

impl<T: Send> Channel<T> for Singleshot<T> {
    fn recv(&mut self) -> Pin<Box<dyn Future<Output = Result<T, ActorError>> + Send + '_>> {
        Box::pin(SingleshotFuture {
            value: &mut self.value,
        })
    }
}

struct SingleshotFuture<'a, T> {
    value: &'a mut Option<T>,
}

impl<T> Future for SingleshotFuture<'_, T> {
    type Output = Result<T, ActorError>;

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.value.take() {
            Some(res) => Poll::Ready(Ok(res)),
            None => Poll::Ready(Err(ActorError::ActorClosed)),
        }
    }
}
