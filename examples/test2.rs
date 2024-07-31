#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
use pakka::messages;
pub struct Test {
    frame: u32,
}
#[automatically_derived]
impl ::core::clone::Clone for Test {
    #[inline]
    fn clone(&self) -> Test {
        Test {
            frame: ::core::clone::Clone::clone(&self.frame),
        }
    }
}
impl Peb for Test {
    fn update(&mut self) -> u32 {
        self.frame += 1;
        self.frame
    }
}
pub trait Peb: Send + Sync + Clone + 'static {
    fn update(&mut self) -> u32;
}
pub struct Updater<T: Peb> {
    peb: T,
}
mod updater_t {
    use pakka::channel::mpsc::{channel, Receiver, Sender};
    use std::marker::PhantomData;
    use futures::stream::*;
    use futures::FutureExt;
    use super::*;
    impl<T: Peb> pakka::Actor for Updater<T> {
        type Ask = UpdaterAskMessage<T>;
        type Tell = UpdaterTellMessage<T>;
    }
    pub struct UpdaterHandle<T: Peb> {
        sender: Sender<Message<T>>,
    }
    #[automatically_derived]
    impl<T: ::core::clone::Clone + Peb> ::core::clone::Clone for UpdaterHandle<T> {
        #[inline]
        fn clone(&self) -> UpdaterHandle<T> {
            UpdaterHandle {
                sender: ::core::clone::Clone::clone(&self.sender),
            }
        }
    }
    #[automatically_derived]
    impl<T: ::core::fmt::Debug + Peb> ::core::fmt::Debug for UpdaterHandle<T> {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field1_finish(
                f,
                "UpdaterHandle",
                "sender",
                &&self.sender,
            )
        }
    }
    pub enum UpdaterAskMessage<T: Peb> {
        Update(tokio::sync::oneshot::Sender<u32>),
        #[doc(hidden)]
        __Phantom(PhantomData<T>),
    }
    #[automatically_derived]
    impl<T: ::core::fmt::Debug + Peb> ::core::fmt::Debug for UpdaterAskMessage<T> {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match self {
                UpdaterAskMessage::Update(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "Update",
                        &__self_0,
                    )
                }
                UpdaterAskMessage::__Phantom(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "__Phantom",
                        &__self_0,
                    )
                }
            }
        }
    }
    pub enum UpdaterTellMessage<T: Peb> {
        #[doc(hidden)]
        __Phantom(PhantomData<T>),
    }
    #[automatically_derived]
    impl<T: ::core::fmt::Debug + Peb> ::core::fmt::Debug for UpdaterTellMessage<T> {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match self {
                UpdaterTellMessage::__Phantom(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "__Phantom",
                        &__self_0,
                    )
                }
            }
        }
    }
    #[automatically_derived]
    impl<T: ::core::clone::Clone + Peb> ::core::clone::Clone for UpdaterTellMessage<T> {
        #[inline]
        fn clone(&self) -> UpdaterTellMessage<T> {
            match self {
                UpdaterTellMessage::__Phantom(__self_0) => {
                    UpdaterTellMessage::__Phantom(::core::clone::Clone::clone(__self_0))
                }
            }
        }
    }
    type Message<T: Peb> = pakka::Message<
        <Updater<T> as pakka::Actor>::Ask,
        <Updater<T> as pakka::Actor>::Tell,
    >;
    impl<T: Peb> Updater<T> {
        pub fn run(mut self) -> UpdaterHandle<T> {
            let (tx, mut rx) = channel::<Message<T>>(100);
            tokio::spawn(async move {
                let mut ctx = pakka::ActorContext::<Self>::new(Box::new(rx));
                loop {
                    let mut remove_index: Option<usize> = None;
                    let msg = ctx.rx.recv().await;
                    match msg {
                        Ok(msg) => self.handle_message(msg, &mut ctx).await,
                        Err(err) => {
                            break;
                        }
                    }
                }
            });
            UpdaterHandle::new(tx)
        }
        fn exit(&self) {
            {
                ::std::io::_print(format_args!("{0} actor task exiting\n", "Updater"));
            };
        }
        async fn handle_message(
            &mut self,
            msg: Message<T>,
            mut _ctx: &mut pakka::ActorContext<Self>,
        ) {
            match msg {
                pakka::Message::Ask(ask_msg) => {
                    self.handle_asks(ask_msg, &mut _ctx).await
                }
                pakka::Message::Tell(tell_msg) => {
                    self.handle_tells(tell_msg, &mut _ctx).await
                }
            }
        }
        async fn handle_asks(
            &mut self,
            msg: UpdaterAskMessage<T>,
            mut _ctx: &mut pakka::ActorContext<Self>,
        ) {
            match msg {
                UpdaterAskMessage::Update(resp) => {
                    let result = self.update(&mut _ctx);
                    let _ = resp.send(result);
                }
                UpdaterAskMessage::__Phantom(_) => {}
            }
        }
        async fn handle_tells(
            &mut self,
            msg: UpdaterTellMessage<T>,
            mut _ctx: &mut pakka::ActorContext<Self>,
        ) {
            match msg {
                UpdaterTellMessage::__Phantom(_) => {}
            }
        }
    }
    impl<T: Peb> UpdaterHandle<T> {
        pub fn new(sender: Sender<Message<T>>) -> Self {
            Self { sender }
        }
        pub async fn update(&self) -> Result<(u32), pakka::ActorError> {
            let (tx, rx) = tokio::sync::oneshot::channel();
            self.sender.send(pakka::Message::Ask(UpdaterAskMessage::Update(tx))).await?;
            rx.await.map_err(Into::into)
        }
    }
}
pub use updater_t::*;
#[allow(dead_code)]
impl<T: Peb> Updater<T> {
    fn update(&mut self, _ctx: &mut pakka::ActorContext<Self>) -> u32 {
        self.peb.update()
    }
}
fn main() {
    let body = async {
        {
            ::std::io::_print(format_args!("Frame {0}\n", frame));
        }
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
