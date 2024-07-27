
use std::fmt;
pub mod channel;
pub use self::channel::mpsc::*;


use channel::mpsc;
pub use pakka_macro::messages;

pub struct ActorCtx<'a, T> {
    pub rx: &'a mut mpsc::Receiver<T> 
}

impl<'a, T> ActorCtx<'a, T> {
    pub fn shut_down_actor(&mut self) {
        self.rx.close();
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

async fn test() {
    let (tx, mut rx) = mpsc::channel::<u32>(100);
    let asd = rx.recv().await;

    if let Some(msg) = rx.recv().await {
        let mut ctx = ActorCtx { rx: &mut rx };
        borrowing_function(msg, &mut ctx);
    }

}


fn borrowing_function<T>(msg: T, ctx: &mut ActorCtx<'_, T>) {
    // If some condition is met, close the receiver
    ctx.shut_down_actor();
}

#[tokio::test]
async fn test2() -> Result<(), ActorError> {
    let (tx, mut rx) = tokio::sync::mpsc::channel::<u32>(100);

    if let Err(err) = tx.try_send(12) {
        println!("Error: {:?}", err);
    }

    if let Err(err) = tx.send(12).await {
        println!("Error: {:?}", err);
    }

    if let Some(msg) = rx.recv().await {
        borrowing_function2(msg, rx);
    }

    let (tx, rx) = tokio::sync::oneshot::channel();
    _ = tx.send(12);

    rx.await?;

    //rx.recv().await.
    Ok(())
}


fn borrowing_function2<T>(msg: T, mut channel: impl Channel) {
    // If some condition is met, close the receiver
    println!("Closing channel!");
    channel.close_channel();
}

pub trait Channel {
    fn close_channel(&mut self);
}


impl<T> Channel for tokio::sync::mpsc::Receiver<T> {
    fn close_channel(&mut self) {
        self.close();
    }
}


