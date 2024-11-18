use std::future::{ready, Ready};
use std::{any::Any, fmt, future::Future, marker::PhantomData, pin::Pin, time::Duration};
pub mod channel;

//mod actor;
mod channels;
pub use self::channel::mpsc::*;
//pub use actor::*;
pub use channels::delayed_message::DelayedMessage;
pub use channels::interval_channel::{IntervalMessage, Intervaller};
pub use channels::singleshot::Singleshot;

use channel::mpsc;
use futures::FutureExt;
pub use pakka_macro::messages;
use tokio::sync::{broadcast, oneshot};

#[derive(Debug)]
pub enum Message<Ask, Tell> {
    Ask(Ask),
    Tell(Tell),
}

pub trait ActorHandle<T> {
    fn new(tx: Box<dyn ChannelSender<T>>) -> Self;
}
pub trait UnboundedActorHandle<T> {
    fn new(tx: Box<dyn UnboundedChannelSender<T>>) -> Self;
}

type UnboundedActorSender<T> = Box<dyn UnboundedChannelSender<T>>;
type ActorSender<T> = Box<dyn ChannelSender<T>>;
type ActorReceiver<T> = Box<dyn Channel<T>>;

pub trait Actor: Sized + Send + 'static {
    type Ask: Send;
    type Tell: Clone + Send + fmt::Debug;
    type Handle: ActorHandle<Message<Self::Ask, Self::Tell>> + fmt::Debug;
    type UnboundedHandle: UnboundedActorHandle<Message<Self::Ask, Self::Tell>> + fmt::Debug;

    fn handle_asks(
        &mut self,
        msg: Self::Ask,
        _ctx: &mut ActorContext<Self>,
    ) -> impl Future<Output = ()> + Send;
    fn handle_tells(
        &mut self,
        msg: Self::Tell,
        _ctx: &mut ActorContext<Self>,
    ) -> impl Future<Output = ()> + Send;

    fn handle_message(
        &mut self,
        msg: Message<Self::Ask, Self::Tell>,
        mut _ctx: &mut ActorContext<Self>,
    ) -> impl Future<Output = ()> + Send {
        async move {
            match msg {
                Message::Ask(ask_msg) => self.handle_asks(ask_msg, &mut _ctx).await,
                Message::Tell(tell_msg) => self.handle_tells(tell_msg, &mut _ctx).await,
            }
        }
    }

    fn run(self) -> Self::Handle {
        self.run_bounded(50)
    }

    fn run_bounded(self, capacity: usize) -> Self::Handle {
        //let (tx, rx) = channel::<<Self as ActorMessage>::Message>(100);
        //self.run_with_channels(Box::new(tx), Box::new(rx), vec![]);
        self.run_with_channels(vec![])
    }

    fn run_unbounded(
        self,
        extra_channel_receivers: Vec<Box<dyn Channel<<Self as Actor>::Tell>>>,
    ) -> Self::UnboundedHandle {
        let (tx, rx) = crate::mpsc::unbounded_channel::<<Self as ActorMessage>::Message>();
        let rx = Box::new(rx);
        let tx = Box::new(tx);
        self.run_task(rx, extra_channel_receivers);
        Self::UnboundedHandle::new(tx)
    }

    fn run_with_channels(
        self,
        extra_channel_receivers: Vec<Box<dyn Channel<<Self as Actor>::Tell>>>,
    ) -> Self::Handle {
        let (tx, rx) = crate::mpsc::channel::<<Self as ActorMessage>::Message>(100);
        let rx = Box::new(rx);
        let tx = Box::new(tx);
        self.run_task(rx, extra_channel_receivers);
        Self::Handle::new(tx)
    }

    fn run_task(
        mut self,
        rx: ActorReceiver<<Self as ActorMessage>::Message>,
        mut extra_channel_receivers: Vec<Box<dyn Channel<<Self as Actor>::Tell>>>,
    ) {
        tokio::spawn(async move {
            let mut ctx = ActorContext::<Self> {
                rx,
                extra_rxs: vec![],
                kill_flag: false,
            };

            loop {
                // Move any added extra channels to be polled
                if !ctx.extra_rxs.is_empty() {
                    extra_channel_receivers.append(&mut ctx.extra_rxs);
                }

                let mut remove_index: Option<usize> = None;
                //If we should poll multiple channels
                if !extra_channel_receivers.is_empty() {
                    let future = futures::future::select_all(
                        extra_channel_receivers
                            .iter_mut()
                            .map(|channel| channel.recv().boxed()),
                    );

                    tokio::select! {
                        msg = ctx.rx.recv() => {
                            //let mut ctx = pakka::ActorCtx::new(rx);
                            match msg {
                                Ok(msg) => self.handle_message(msg, &mut ctx).await,
                                Err(_) => {
                                    // The channel has closed, exit the loop
                                    break;
                                }
                            }
                        },
                        (result, index, _whatsthis) = future => {

                            match result {
                                Ok(msg) => self.handle_tells(msg, &mut ctx).await,
                                Err(_) => remove_index = Some(index),
                            }
                        }
                    }
                    if let Some(index) = remove_index {
                        extra_channel_receivers.swap_remove(index);
                    }
                } else {
                    let msg = ctx.rx.recv().await;
                    match msg {
                        Ok(msg) => self.handle_message(msg, &mut ctx).await,
                        Err(_) => {
                            // The channel has closed, exit the loop
                            break;
                        }
                    }
                }

                if ctx.kill_flag {
                    break;
                }
            }
        });
    }
}

pub trait ActorMessage: Actor {
    type Message: Send;
}

impl<T: Actor> ActorMessage for T {
    type Message = Message<T::Ask, T::Tell>;
}

pub struct ActorContext<A>
where
    A: Actor,
{
    pub rx: Box<dyn Channel<Message<A::Ask, A::Tell>>>,
    pub extra_rxs: Vec<Box<dyn Channel<A::Tell>>>,
    pub kill_flag: bool,
}

impl<A: Actor + ActorMessage> ActorContext<A> {
    pub fn new(rx: Box<dyn Channel<Message<A::Ask, A::Tell>>>) -> Self {
        Self {
            rx,
            extra_rxs: vec![],
            kill_flag: false,
        }
    }

    pub fn shut_down_actor(&mut self) {
        self.kill_flag = true;
    }

    //* Takes Tell that will be processed, once */
    pub fn tell(&mut self, msg: A::Tell) {
        let msg = Singleshot::new(msg);
        self.extra_rxs.push(Box::new(msg));
    }

    pub fn delayed_tell(&mut self, msg: A::Tell, delay: std::time::Duration) {
        let msg = DelayedMessage {
            value: Some(msg),
            delay: Box::pin(tokio::time::sleep(delay)),
        };
        self.extra_rxs.push(Box::new(msg));
    }

    //* Adds extra channels that'll be used to receive Tells from */
    pub fn add_channel(&mut self, channel: Box<dyn Channel<A::Tell>>) {
        self.extra_rxs.push(channel);
    }
}

#[derive(Debug)]
pub enum ActorError {
    ActorClosed,
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

pub trait ActorChannelSender<'a, T>: Send + Sync + fmt::Debug {
    type SendFuture: Future<Output = Result<(), ActorError>> + Send + 'a;
    fn send(&'a self, msg: T) -> Self::SendFuture;
    fn clone_box(&self) -> Box<dyn ActorChannelSender<'a, T, SendFuture = Self::SendFuture>>;
}

impl<'a, T: Send + 'static> ActorChannelSender<'a, T> for mpsc::Sender<T> {
    type SendFuture = Pin<Box<dyn Future<Output = Result<(), ActorError>> + Send + 'a>>;

    fn send(&'a self, msg: T) -> Self::SendFuture {
        Box::pin(async move { self.send(msg).await.map_err(|e| e.into()) })
    }

    fn clone_box(&self) -> Box<dyn ActorChannelSender<'a, T, SendFuture = Self::SendFuture>> {
        Box::new(self.clone())
    }
}

impl<'a, T: Send + 'static> ActorChannelSender<'a, T> for mpsc::UnboundedSender<T> {
    type SendFuture = Ready<Result<(), ActorError>>;

    fn send(&self, msg: T) -> Self::SendFuture {
        ready(self.send(msg).map_err(|e| e.into()))
    }

    fn clone_box(&self) -> Box<dyn ActorChannelSender<'a, T, SendFuture = Self::SendFuture>> {
        Box::new(self.clone())
    }
}

// Sending is non async, but also with no back pressure
pub trait UnboundedChannelSender<T>: fmt::Debug {
    fn send(&self, msg: T) -> Result<(), ActorError>;
    fn clone_box(&self) -> Box<dyn UnboundedChannelSender<T>>;
}
impl<T> Clone for Box<dyn UnboundedChannelSender<T>> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}
impl<T: Send + 'static> UnboundedChannelSender<T> for mpsc::UnboundedSender<T> {
    fn send(&self, msg: T) -> Result<(), ActorError> {
        self.send(msg).map_err(|e| e.into())
    }

    fn clone_box(&self) -> Box<dyn UnboundedChannelSender<T>> {
        Box::new(self.clone())
    }
}

///
/// ChannelSender trait for tokio::sync::mpsc::Sender<T> and such.
/// Maybe unnecessary, but wanted to experiment with alternative channels easily
///
pub trait ChannelSender<T>: Send + Sync + fmt::Debug {
    fn send(&self, msg: T) -> Pin<Box<dyn Future<Output = Result<(), ActorError>> + Send + '_>>;
    fn clone_box(&self) -> Box<dyn ChannelSender<T>>;
}

impl<T> Clone for Box<dyn ChannelSender<T>> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

impl<T: Send + 'static> ChannelSender<T> for mpsc::Sender<T> {
    fn send(&self, msg: T) -> Pin<Box<dyn Future<Output = Result<(), ActorError>> + Send + '_>> {
        //let sender = self.clone(); // Clone the sender
        Box::pin(async move { self.send(msg).await.map_err(|e| e.into()) })
    }

    fn clone_box(&self) -> Box<dyn ChannelSender<T>> {
        Box::new(self.clone())
    }
}

impl<T: Send + 'static> ChannelSender<T> for kanal::AsyncSender<T> {
    fn send(&self, msg: T) -> Pin<Box<dyn Future<Output = Result<(), ActorError>> + Send + '_>> {
        //let sender = self.clone(); // Clone the sender
        Box::pin(async move {
            match self.send(msg).await {
                Ok(_) => Ok(()),
                Err(_) => Err(ActorError::ActorClosed),
            }
        })
    }

    fn clone_box(&self) -> Box<dyn ChannelSender<T>> {
        Box::new(self.clone())
    }
}

///
/// Abstraction for channel receiver, so there can be many forms of receivers
///
pub trait Channel<T>: Send {
    fn recv(&mut self) -> Pin<Box<dyn Future<Output = Result<T, ActorError>> + Send + '_>>;
}

impl<T: Send> Channel<T> for mpsc::Receiver<T> {
    fn recv(&mut self) -> Pin<Box<dyn Future<Output = Result<T, ActorError>> + Send + '_>> {
        Box::pin(async move {
            match self.recv().await {
                Some(value) => Ok(value),
                None => Err(ActorError::ActorClosed),
            }
        })
    }
}

impl<T: Send> Channel<T> for mpsc::UnboundeReceiver<T> {
    fn recv(&mut self) -> Pin<Box<dyn Future<Output = Result<T, ActorError>> + Send + '_>> {
        Box::pin(async move {
            match self.recv().await {
                Some(value) => Ok(value),
                None => Err(ActorError::ActorClosed),
            }
        })
    }
}

impl<T: Send + Clone> Channel<T> for broadcast::Receiver<T> {
    fn recv(&mut self) -> Pin<Box<dyn Future<Output = Result<T, ActorError>> + Send + '_>> {
        Box::pin(async move {
            match broadcast::Receiver::recv(self).await {
                Ok(value) => Ok(value),
                Err(err) => match err {
                    tokio::sync::broadcast::error::RecvError::Closed => {
                        Err(ActorError::ActorClosed)
                    }
                    tokio::sync::broadcast::error::RecvError::Lagged(x) => {
                        eprint!("Lagged!: {}", x);
                        Err(ActorError::ActorClosed) //TODO: FIX
                    }
                },
            }
        })
    }
}

impl<T: Send> Channel<T> for kanal::AsyncReceiver<T> {
    fn recv(&mut self) -> Pin<Box<dyn Future<Output = Result<T, ActorError>> + Send + '_>> {
        Box::pin(async move {
            match kanal::AsyncReceiver::recv(self).await {
                Ok(value) => Ok(value),
                Err(_) => Err(ActorError::ActorClosed),
            }
        })
    }
}
