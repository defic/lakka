
use std::{any::Any, fmt, future::Future, marker::PhantomData, pin::Pin};
pub mod channel;
pub use self::channel::mpsc::*;


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

pub trait Actor: Sized + Send + 'static {
    type Ask: Send;
    type Tell: Clone + Send;
    type Handle: ActorHandle<Message<Self::Ask, Self::Tell>> + fmt::Debug;

    fn handle_asks(&mut self, msg: Self::Ask, _ctx: &mut ActorContext<Self>) -> impl Future<Output = ()> + Send;
    fn handle_tells(&mut self, msg: Self::Tell, _ctx: &mut ActorContext<Self>) -> impl Future<Output = ()> + Send;

    fn handle_message(&mut self, msg: Message<Self::Ask, Self::Tell>, mut _ctx: &mut ActorContext<Self>) -> impl Future<Output = ()> + Send {
        async move {
            match msg {
                Message::Ask(ask_msg) => self.handle_asks(ask_msg, &mut _ctx).await,
                Message::Tell(tell_msg) => self.handle_tells(tell_msg, &mut _ctx).await,
            }
        } 
    }

    fn run(self) -> Self::Handle {
        self.run_with_channels(vec![])
    }

    fn run_with_channels(mut self, mut channels: Vec<Box<dyn Channel<<Self as Actor>::Tell>>>) -> Self::Handle { 
        let (tx, mut rx) = channel::<<Self as ActorMessage>::Message>(100);

        tokio::spawn(async move {
            let mut ctx = ActorContext::<Self>{
                rx: Box::new(rx),
                extra_rxs: vec![],
                kill_flag: false, 
            };

            loop {
                // Move any added extra channels to be polled
                if !ctx.extra_rxs.is_empty() {
                    channels.append(&mut ctx.extra_rxs);
                }

                let mut remove_index: Option<usize> = None;
                //If we should poll multiple channels
                if !channels.is_empty() {
                    let future = futures::future::select_all(
                        channels
                            .iter_mut()
                            .map(|channel| channel.recv().boxed()),
                    );

                    tokio::select! {
                        msg = ctx.rx.recv() => {
                            //let mut ctx = pakka::ActorCtx::new(rx);
                            match msg {
                                Ok(msg) => self.handle_message(msg, &mut ctx).await,
                                Err(err) => {
                                    // The channel has closed, exit the loop
                                    break;
                                }
                            }
                        },
                        (result, index, _) = future => {
                            match result {
                                Ok(msg) => self.handle_tells(msg, &mut ctx).await,
                                Err(error) => {
                                    println!("Channel died, removing index: {}, error: {}", index, error);
                                    remove_index = Some(index);
                                }
                            }
                        }
                    }
                    if let Some(index) = remove_index {
                        println!("Channel died, removing index: {}, length is: {}", index, ctx.extra_rxs.len() );
                        channels.remove(index);
                        println!("Removed, now length {}", ctx.extra_rxs.len() );
                    }
                }
                else {
                    let msg = ctx.rx.recv().await;
                    match msg {
                        Ok(msg) => self.handle_message(msg, &mut ctx).await,
                        Err(err) => {
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

        Self::Handle::new(Box::new(tx))
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

    //* Takes Tell that will be processed later, once */
    pub fn tell(&mut self, msg: A::Tell) {
        self.extra_rxs.push(Box::new(Some(msg)));
    }

    //* Adds extra channels that'll be used to receive Tells from */
    pub fn add_channel(&mut self, channel: Box<dyn Channel<A::Tell>>) {
        self.extra_rxs.push(channel);
    }
}

struct GenericActor<T> {
    field: T
}

/* 
impl <T> Actor for GenericActor<T> {
    type Ask = Asks;
    type Tell = Tells;
    type Handle = u8;
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
*/


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

/* 
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
    type Handle = u8;
}
    */

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

pub trait ChannelSender<T>: Send + Sync + fmt::Debug {
    fn send(&self, msg: T) -> Pin<Box<dyn Future<Output = Result<(), ActorError>> + Send + '_>>;
    fn clone_box(&self) -> Box<dyn ChannelSender<T>>;
}

impl <T> Clone for Box<dyn ChannelSender<T>> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

// Implement ChannelSender for tokio::sync::mpsc::Sender
impl<T: Send + 'static> ChannelSender<T> for mpsc::Sender<T> {
    fn send(&self, msg: T) -> Pin<Box<dyn Future<Output = Result<(), ActorError>> + Send + '_>> {
        //let sender = self.clone(); // Clone the sender
        Box::pin(async move {
            self.send(msg).await.map_err(|e| e.into())
        })
    }

    fn clone_box(&self) -> Box<dyn ChannelSender<T>> {
        Box::new(self.clone())
    }
}

/*
pub struct Test<T> {
    sender: Vec<Box<dyn ChannelSender<T>>>
}
*/

pub trait Channel<T>: Send {
    fn recv(&mut self) -> Pin<Box<dyn Future<Output = Result<T, ActorError>> + Send + '_>>;
}


pub struct PostponeChannel<T>(Option<T>);

impl <T: Send> PostponeChannel<T> {
    pub fn new(content: T) -> Self {
        Self (Some(content))
    }
}

impl<T: Send> Channel<T> for PostponeChannel<T> {
    fn recv(&mut self) -> Pin<Box<dyn Future<Output = Result<T, ActorError>> + Send + '_>> {
        let res = self.0.take();
        let res = match res {
            Some(value) => Ok(value),
            None => Err(ActorError::ActorClosed),
        };
        Box::pin(std::future::ready(res))
    }
}

impl<T: Send> Channel<T> for Option<T> {
    fn recv(&mut self) -> Pin<Box<dyn Future<Output = Result<T, ActorError>> + Send + '_>> {
        let res = self.take();
        let res = match res {
            Some(value) => Ok(value),
            None => Err(ActorError::ActorClosed),
        };
        Box::pin(std::future::ready(res))
    }
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

impl<T: Send + Clone> Channel<T> for broadcast::Receiver<T> {
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

impl <Message: Send> Interval<Message>{
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

