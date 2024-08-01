#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
use pakka::messages;
struct Actor {}
mod actor {
    use pakka::channel::mpsc::{channel, Receiver, Sender};
    use std::marker::PhantomData;
    use futures::stream::*;
    use futures::FutureExt;
    use super::*;
    impl pakka::Actor for Actor {
        type Ask = ActorAskMessage;
        type Tell = ActorTellMessage;
        type Handle = ActorHandle<Self>;
        async fn handle_asks(
            &mut self,
            msg: Self::Ask,
            mut _ctx: &mut pakka::ActorContext<Self>,
        ) {
            match msg {
                ActorAskMessage::Get(resp) => {
                    let result = self.get(&mut _ctx);
                    let _ = resp.send(result);
                }
            }
        }
        async fn handle_tells(
            &mut self,
            msg: Self::Tell,
            mut _ctx: &mut pakka::ActorContext<Self>,
        ) {
            match msg {
                ActorTellMessage::Test() => {
                    self.test(&mut _ctx);
                }
            }
        }
    }
    pub struct ActorHandle {
        sender: Box<dyn pakka::ChannelSender<Message>>,
    }
    #[automatically_derived]
    impl ::core::clone::Clone for ActorHandle {
        #[inline]
        fn clone(&self) -> ActorHandle {
            ActorHandle {
                sender: ::core::clone::Clone::clone(&self.sender),
            }
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for ActorHandle {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field1_finish(
                f,
                "ActorHandle",
                "sender",
                &&self.sender,
            )
        }
    }
    impl pakka::ActorHandle<Message> for ActorHandle {
        fn new(tx: Box<dyn pakka::ChannelSender<Message>>) -> Self {
            Self { sender: tx }
        }
    }
    pub enum ActorAskMessage {
        Get(tokio::sync::oneshot::Sender<u32>),
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for ActorAskMessage {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match self {
                ActorAskMessage::Get(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "Get",
                        &__self_0,
                    )
                }
            }
        }
    }
    pub enum ActorTellMessage {
        Test(),
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for ActorTellMessage {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::write_str(f, "Test")
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for ActorTellMessage {
        #[inline]
        fn clone(&self) -> ActorTellMessage {
            ActorTellMessage::Test()
        }
    }
    type Message = pakka::Message<
        <Actor as pakka::Actor>::Ask,
        <Actor as pakka::Actor>::Tell,
    >;
    impl Actor {
        fn exit(&self) {
            {
                ::std::io::_print(format_args!("{0} actor task exiting\n", "Actor"));
            };
        }
    }
    impl ActorHandle {
        pub fn new(sender: Sender<Message>) -> Self {
            Self { sender }
        }
        pub async fn test(&self) -> Result<(), pakka::ActorError> {
            self.sender.send(pakka::Message::Tell(ActorTellMessage::Test())).await?;
            Ok(())
        }
        pub async fn get(&self) -> Result<(u32), pakka::ActorError> {
            let (tx, rx) = tokio::sync::oneshot::channel();
            self.sender.send(pakka::Message::Ask(ActorAskMessage::Get(tx))).await?;
            rx.await.map_err(Into::into)
        }
    }
}
pub use actor::*;
#[allow(dead_code)]
impl Actor {
    fn test(&self, _ctx: &mut pakka::ActorContext<Self>) {
        {
            ::std::io::_print(format_args!("test\n"));
        };
    }
    fn get(&self, _ctx: &mut pakka::ActorContext<Self>) -> u32 {
        15
    }
}
fn main() {
    let body = async {
        let test = Actor {};
    };
    #[allow(clippy::expect_used, clippy::diverging_sub_expression)]
    {
        return tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed building the Runtime")
            .block_on(body);
    }
}
