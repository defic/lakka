
use std::{fmt, future::Future, marker::PhantomData};
pub mod channel;
pub use self::channel::mpsc::*;


use channel::mpsc;
pub use pakka_macro::messages;

pub struct ActorCtx<C, T> 
where 
    C: Channel<T>
{
    pub rx: C,
    pub kill_flag: bool,
    _t: std::marker::PhantomData<T>
}

impl<C, T> ActorCtx<C, T>
where 
    C: Channel<T>
{
    pub fn new(rx: C) -> Self {
        Self {
            rx, kill_flag: false, _t: Default::default()
        }
    }

    pub fn shut_down_actor(&mut self) {
        self.kill_flag = true;
    }
}

#[derive(Debug)]
pub enum ActorError {
    ActorClosed
}

impl std::error::Error for ActorError {}

impl fmt::Display for ActorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ActorError::ActorClosed => write!(f, "Actor is closed"),
        }
    }
}

impl<T> From<SendError<T>> for ActorError {
    fn from(_: SendError<T>) -> Self {
        ActorError::ActorClosed
    }
}

impl From<RecvError> for ActorError {
    fn from(_: RecvError) -> Self {
        ActorError::ActorClosed
    }
}

async fn test()
{
    let (tx, mut rx) = channel::<ConcreteType>(100);
    let asd = rx.recv().await;

    if let Some(msg) = rx.recv().await {
        let mut ctx = ActorCtx::new(rx);
        borrowing_function(msg, &mut ctx);
    }

    let k = SomeThing {
        sender: tx
    };
}


fn borrowing_function<T>(msg: T, ctx: &mut ActorCtx<impl Channel<T>, T>) {
    // If some condition is met, close the receiver
    ctx.shut_down_actor();


}

pub struct Assh<T, C: ChannelSender<T>> {
    pub tx: C,
    _t: std::marker::PhantomData<T>
}
pub struct ConcreteType {}

#[derive(Clone)]
pub struct SomeThing<S>
where 
    S: ChannelSender<ConcreteType>
{
    pub sender: S
}

pub trait ChannelSender<T> {
    fn send(&self, msg: T) -> impl Future<Output = Result<(), ActorError>>;
}

impl <T> ChannelSender<T> for tokio::sync::mpsc::Sender<T> {
    async fn send(&self, msg: T) -> Result<(), ActorError> {
        Self::send(self, msg).await.map_err(|e| e.into())
    }
}


pub trait Channel<T> {
    fn recv(&mut self) -> impl Future<Output = Result<T, ActorError>> + Send;
}


impl<T: Send> Channel<T> for tokio::sync::mpsc::Receiver<T> {

    async fn recv(&mut self) -> Result<T, ActorError> {
        match self.recv().await {
            Some(value) => Ok(value),
            None => Err(ActorError::ActorClosed),
        }
    }
}

impl Channel<tokio::time::Instant> for tokio::time::Interval {
    async fn recv(&mut self) -> Result<tokio::time::Instant, ActorError> {
        let res = self.tick().await;
        Ok(res)
    }
}

struct Interval<Message: Clone> {
    interval: tokio::time::Interval,
    message: Message,
}

/* 
impl <Message: Clone> Channel<Message> for Interval<Message> {

    async fn recv(&mut self) -> Result<Message, ActorError> {
        let _ = self.interval.tick().await;
        Ok(self.message.clone())
    }
}
*/