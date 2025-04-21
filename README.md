# Lakka

## Description
Actor system inspired by Alice Ryhls [Actors with Tokio](https://ryhl.io/blog/actors-with-tokio/) blog post, improved with macros to reduce boilerplate and create a more friendlier user experience. The functionality of this library is focused on the actor message ergonomics, and there are no built in actor supervision, support for distributed actors or other more sophisticated features.

## Installation
```bash
cargo add lakka
```

## Usage

Adding `#[messages]` macro to an impl block will make all the functions with `&self` and `&mut self` callable from an actor handle. Calling `run()` returns an actor handle, that can be cloned and passed around.

```rust
use std::time::Duration;
use lakka::*;

#[derive(Default)]
struct Counter {
    counter: i32,
}

//Only macro needed to use lakka. Will generate CounterHandle, necessary enums for messages and implement Actor trait etc.
#[messages]
impl Counter {
    // modify actor state
    fn inc(&mut self) {
        self.counter += 1;
    }

    fn get(&self) -> i32 {
        self.counter
    }
}

#[tokio::main]
async fn main() {
    // Run will run the actor in a tokio task.
    let counter_handle = Counter::default().run();
    // Functions with no return values from the actor are considered as "Actor Tell" message.
    _ = counter_handle.inc().await;
    _ = counter_handle.inc().await;
    // "Actor Ask" message when there is a return value
    let state = counter_handle.get().await.unwrap();
    assert_eq!(state, 2);

    let second_handle = counter_handle.clone();
    tokio::spawn(async move {
        loop {
            second_handle.inc().await.unwrap();
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    });

    tokio::time::sleep(Duration::from_secs(1)).await;
    println!("Counter value: {}", counter_handle.get().await.unwrap());
}
```

## Features
- `#[messages]` macro for bounded Actor, this is the typical usage. Uses `tokio::sync::mpsc::channel` to deliver messages to the actor. `.run()` defaults to channel size 100. With `run_bounded` the channel size can be customized.
- `#[messages(unbounded)]` for "unbounded" actor, when unbounded channel for messages is needed. Uses `tokio::sync::mpsc::unbounded_channel`. The main difference that sending of the messages are not awaited. This can be useful f.e. when Actor implements `Drop` (non async) and needs to signal another actor through it's handle. 
- Actor can receive messages from multiple sources naturally when an actor handle is cloned, but in addition to this the actor can have additional ActorTellChannels to receive messages from, that can be registered either with `run_with_channels` function, or in `#[messages]` block function by calling `_ctx.extra_rxs.push(receiver)`. This can be useful with broadcast type scenario, like chats.

Check the examples for more examples!

## Behind the scenes

The `#[messages]` macro in the above example generates code to implement lakka::Actor for Counter, `struct CounterHandle`, an `enum CounterAskMessage` with an enum variant for each function that returns a value, and an `enum CounterTellMessage` with a variant for each function that doesn't return a value.

```rust
//Generated code:
mod counter {
    use super::*;
    impl lakka::BoundedActor for Counter {
        type Handle = CounterHandle;
    }
    impl lakka::Actor for Counter {
        type Ask = CounterAskMessage;
        type Tell = CounterTellMessage;
        async fn handle_asks(&mut self, msg: Self::Ask, mut _ctx: &mut lakka::ActorContext<Self>) {
            match msg {
                CounterAskMessage::Get(resp) => {
                    let result = self.get(&mut _ctx);
                    let _ = resp.send(result);
                }
            }
        }
        async fn handle_tells(
            &mut self,
            msg: Self::Tell,
            mut _ctx: &mut lakka::ActorContext<Self>,
        ) {
            match msg {
                CounterTellMessage::Inc() => {
                    self.inc(&mut _ctx);
                }
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct CounterHandle {
        sender: Box<dyn lakka::ChannelSender<CounterMessage>>,
    }
    impl CounterHandle {
        pub async fn inc(&self) -> Result<(), lakka::ActorError> {
            self.sender
                .send(lakka::Message::Tell(CounterTellMessage::Inc()))
                .await?;
            Ok(())
        }
        pub async fn get(&self) -> Result<(i32), lakka::ActorError> {
            let (tx, rx) = tokio::sync::oneshot::channel();
            self.sender
                .send(lakka::Message::Ask(CounterAskMessage::Get(tx)))
                .await?;
            rx.await.map_err(Into::into)
        }
    }
    impl lakka::ActorHandle<CounterMessage> for CounterHandle {
        fn new(tx: Box<dyn lakka::ChannelSender<CounterMessage>>) -> Self {
            Self { sender: tx }
        }
    }
    #[derive(Debug)]
    pub enum CounterAskMessage {
        Get(tokio::sync::oneshot::Sender<i32>),
    }
    #[derive(Debug, Clone)]
    pub enum CounterTellMessage {
        Inc(),
    }
    type CounterMessage =
        lakka::Message<<Counter as lakka::Actor>::Ask, <Counter as lakka::Actor>::Tell>;
}
pub use counter::*;
#[allow(dead_code)]
impl Counter {
    fn inc(&mut self, _ctx: &mut lakka::ActorContext<Self>) {
        self.counter += 1;
    }
    fn get(&self, _ctx: &mut lakka::ActorContext<Self>) -> i32 {
        self.counter
    }
}
```

## Contributing
Contributions are always welcome!

## License
This project is licensed under either of:

- MIT License
- Apache License, Version 2.0

at your option.
