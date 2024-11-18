use std::{future::Future, pin::Pin, time::Duration};

use crate::{ActorError, Channel};

#[derive(Debug, Clone)]
pub struct IntervalMessage<T> {
    pub msg: T,
    pub counter: u32,
}

impl<T> IntervalMessage<T> {
    pub fn new(msg: T) -> Self {
        Self { msg, counter: 0 }
    }
}

pub struct Intervaller<Variant, Message: Clone> {
    variant: fn(IntervalMessage<Message>) -> Variant,
    msg: Message,
    counter: u32,
    interval: tokio::time::Interval,
    tick_limit: Option<u32>,
}

impl<Variant, Message: Clone> Intervaller<Variant, Message> {
    pub fn new(
        interval: Duration,
        variant: fn(IntervalMessage<Message>) -> Variant,
        msg: Message,
    ) -> Self {
        Self::new_with_limit(interval, variant, msg, None)
    }

    pub fn new_with_limit(
        interval: Duration,
        variant: fn(IntervalMessage<Message>) -> Variant,
        msg: Message,
        tick_limit: Option<u32>,
    ) -> Self {
        let interval = tokio::time::interval(interval);
        Self {
            interval,
            variant,
            msg,
            tick_limit,
            counter: 0,
        }
    }

    pub fn construct(&self) -> Variant {
        let msg = IntervalMessage {
            msg: self.msg.clone(),
            counter: self.counter,
        };

        (self.variant)(msg)
    }
}

impl<Variant, Message: Clone + Send> Channel<Variant> for Intervaller<Variant, Message> {
    fn recv(&mut self) -> Pin<Box<dyn Future<Output = Result<Variant, ActorError>> + Send + '_>> {
        Box::pin(async move {
            if self.tick_limit.map(|limit| self.counter > limit).is_some() {
                return Err(ActorError::ActorClosed);
            }

            let _ = self.interval.tick().await;
            let msg = self.construct();
            self.counter += 1;

            Ok(msg)
        })
    }
}

#[tokio::test]
async fn test() {
    #[derive(Debug, Clone, PartialEq)]
    pub struct Keke(u32);

    pub enum Test {
        Asd(IntervalMessage<Keke>),
    }

    let asd = Intervaller::new(Duration::from_secs(1), Test::Asd, Keke(15));
    let enummi = asd.construct();
    let Test::Asd(IntervalMessage { msg, .. }) = enummi;
    assert_eq!(msg, Keke(15))
}
