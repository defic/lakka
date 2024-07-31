
use std::{fmt, future::Future, marker::PhantomData, pin::Pin};
pub mod channel;
pub use self::channel::mpsc::*;


use channel::mpsc;
use futures::FutureExt;
pub use pakka_macro::messages;
use tokio::sync::broadcast;

#[derive(Debug)]
pub enum Message<Ask, Tell> {
    Ask(Ask),
    Tell(Tell),
}

pub trait Actor {
    type Ask: Send;
    type Tell: Clone + Send;
}

pub trait ActorMessage: Actor {
    type Message: Send;
}

impl<T: Actor> ActorMessage for T {
    type Message = Message<T::Ask, T::Tell>;
}

pub struct ActorContext<A>
where 
    A: Actor + ActorMessage
{
    pub rx: Box<dyn Channel<Message<A::Ask, A::Tell>>>,
    pub extra_rxs: Vec<Box<dyn Channel<A::Tell>>>,
    pub kill_flag: bool,
}

impl <A: Actor + ActorMessage> ActorContext<A> {
    pub fn new(rx: Box<dyn Channel<Message<A::Ask, A::Tell>>>) -> Self {
        Self {
            rx, extra_rxs: vec![], kill_flag: false,
        }
    }

    pub fn shut_down_actor(&mut self) {
        self.kill_flag = true;
    }
}

struct GenericActor<T> {
    field: T
}

impl <T> Actor for GenericActor<T> {
    type Ask = Asks;
    type Tell = Tells;
}

type Msg<T> = Message<<GenericActor<T> as Actor>::Ask, <GenericActor<T> as Actor>::Tell>;

impl<T> GenericActor<T>
where
    T: Send, // or any other bounds required on T
    //GenericActor<T>: Actor + ActorMessage<Message = Message<Asks, Tells>>,
{
    pub fn solves(&self, msg: Msg<T>) {
        match msg {
            Message::Ask(msg) => self.ask(msg),
            Message::Tell(msg) => self.tell(msg),
        }
    }

    pub fn ask(&self, msg: Asks) {
        match msg {
            Asks::One => todo!(),
            Asks::Two => todo!(),
        }
    }

    pub fn tell(&self, msg: Tells) {
        match msg {
            Tells::Four => todo!(),
            Tells::Five => todo!(),
        }
    }
}


pub enum Asks {
    One,
    Two
}
#[derive(Clone)]
pub enum Tells {
    Four,
    Five
}
struct TestActor {}

impl TestActor where TestActor: Actor + ActorMessage {

    pub fn solve(msg: <Self as ActorMessage>::Message) {
        match msg {
            Message::Ask(ask) => Self::ask(ask),
            Message::Tell(tell) => Self::tell(tell),
        }
    }

    pub fn ask(msg: Asks) {
        match msg {
            Asks::One => todo!(),
            Asks::Two => todo!(),
        }
    }

    pub fn tell(msg: <Self as Actor>::Tell) {
        match msg {
            Tells::Four => todo!(),
            Tells::Five => todo!(),
        }
    }
}

impl Actor for TestActor {
    type Ask = Asks;
    type Tell = Tells;
}

pub struct ActorCtx<C, T> 
where 
    C: Channel<T>
{
    pub rx: C,
    pub extra_rxs: Vec<Box<dyn Channel<T>>>,
    pub kill_flag: bool,
    _t: std::marker::PhantomData<T>
}

impl<C, T> ActorCtx<C, T>
where 
    C: Channel<T>
{
    pub fn new(rx: C) -> Self {
        Self {
            rx, kill_flag: false, _t: Default::default(), extra_rxs: Default::default()
        }
    }

    pub fn new_with_extras(rx: C, extra_rxs: Vec<Box<dyn Channel<T>>>) -> Self {
        Self {
            rx, kill_flag: false, _t: Default::default(), extra_rxs
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

async fn test() -> SomeThing<Sender<ConcreteType>> {
    let (tx, mut rx) = channel::<ConcreteType>(100);
    let asd = rx.recv().await;

    let mut ctx = ActorCtx::new(rx);
    testad(&mut ctx).await;


    let k = SomeThing {
        sender: tx
    };
    k
}

async fn many_channels() {
    let mut senders = vec![];
    let mut channels: Vec<Box<dyn Channel<u32>>>  = vec![];
    for _ in 0..10 {
        let ch = channel::<u32>(1);
        senders.push(ch.0);
        channels.push(Box::new(ch.1));
    }

    let (result, index, _) = futures::future::select_all(
        channels
            .iter_mut()
            // note: `FutureExt::boxed` is called here because `select_all`
            //       requires the futures to be pinned
            .map(|listener| listener.recv().boxed()),
    ).await;

    match result {
        Ok(_) => todo!(),
        Err(_) => todo!(),
    }

}

async fn testad<C, T>(ctx: &mut ActorCtx<C, T>) where C: Channel<T> {
    testtt(ctx).await;
}

async fn testtt<C, T>(ctx: &mut ActorCtx<C, T>) where C: Channel<T> { //channel: &impl Channel<T>) {
    if let Ok(msg) = ctx.rx.recv().await {
        borrowing_function(msg, ctx);
    }
}

async fn return_concrete<T: Send + 'static>() -> impl ChannelSender<T> {
    let (tx, mut rx) = channel::<T>(100);
    tx
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

pub trait ChannelSender<T>: Send + Sync {
    fn send(&self, msg: T) -> Pin<Box<dyn Future<Output = Result<(), ActorError>> + Send + '_>>;
}

// Implement ChannelSender for tokio::sync::mpsc::Sender
impl<T: Send + 'static> ChannelSender<T> for mpsc::Sender<T> {
    fn send(&self, msg: T) -> Pin<Box<dyn Future<Output = Result<(), ActorError>> + Send + '_>> {
        //let sender = self.clone(); // Clone the sender
        Box::pin(async move {
            self.send(msg).await.map_err(|e| e.into())
        })
    }
}

pub struct Test<T> {
    sender: Vec<Box<dyn ChannelSender<T>>>
}

pub trait Channel<T>: Send {
    fn recv(&mut self) -> Pin<Box<dyn Future<Output = Result<T, ActorError>> + Send + '_>>;
}

impl<T: Send + 'static> Channel<T> for mpsc::Receiver<T> {
    fn recv(&mut self) -> Pin<Box<dyn Future<Output = Result<T, ActorError>> + Send + '_>> {
        Box::pin(async move {
            match self.recv().await {
                Some(value) => Ok(value),
                None => Err(ActorError::ActorClosed),
            }
        })
    }
}

impl<T: Send + 'static + Clone> Channel<T> for broadcast::Receiver<T> {
    fn recv(&mut self) -> Pin<Box<dyn Future<Output = Result<T, ActorError>> + Send + '_>> {
        Box::pin(async move {
            match broadcast::Receiver::recv(self).await {
                Ok(value) => Ok(value),
                Err(err) => match err {
                    tokio::sync::broadcast::error::RecvError::Closed => Err(ActorError::ActorClosed),
                    tokio::sync::broadcast::error::RecvError::Lagged(x) => {
                        eprint!("Lagged!: {}", x);
                        Err(ActorError::ActorClosed) //TODO: FIX
                    },
                }
            }
        })
    }
}

impl Channel<tokio::time::Instant> for tokio::time::Interval {
    fn recv(&mut self) -> Pin<Box<dyn Future<Output = Result<tokio::time::Instant, ActorError>> + Send + '_>> {
        Box::pin(async move {
            let res = self.tick().await;
            Ok(res)
        })
    }
}

pub struct Interval<Message: Send> {
    pub interval: tokio::time::Interval,
    pub message: Message,
    counter: u32,
}

impl <Message:  Send> Interval<Message>{
    pub fn new(interval: tokio::time::Interval, message: Message) -> Self {
        Self {
            interval,
            message,
            counter: 0
        }
    }
}

impl <Message: Send + Clone> Channel<Message> for Interval<Message> {
    fn recv(&mut self) -> Pin<Box<dyn Future<Output = Result<Message, ActorError>> + Send + '_>> {
        Box::pin(async move {

            self.counter += 1;
            if self.counter > 5 {
                return Err(ActorError::ActorClosed);
            }
            let _ = self.interval.tick().await;
            Ok(self.message.clone())
        })
        
    }
}
/* */

