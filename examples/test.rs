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
    use super::*;
    pub struct UpdaterHandle<T: Peb, S>
    where
        S: pakka::ChannelSender<UpdaterMessage<T>>,
    {
        sender: S,
        _phantom: std::marker::PhantomData<UpdaterMessage<T>>,
    }
    #[automatically_derived]
    impl<T: ::core::clone::Clone + Peb, S: ::core::clone::Clone> ::core::clone::Clone
    for UpdaterHandle<T, S>
    where
        S: pakka::ChannelSender<UpdaterMessage<T>>,
    {
        #[inline]
        fn clone(&self) -> UpdaterHandle<T, S> {
            UpdaterHandle {
                sender: ::core::clone::Clone::clone(&self.sender),
                _phantom: ::core::clone::Clone::clone(&self._phantom),
            }
        }
    }
    #[automatically_derived]
    impl<T: ::core::fmt::Debug + Peb, S: ::core::fmt::Debug> ::core::fmt::Debug
    for UpdaterHandle<T, S>
    where
        S: pakka::ChannelSender<UpdaterMessage<T>>,
    {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field2_finish(
                f,
                "UpdaterHandle",
                "sender",
                &self.sender,
                "_phantom",
                &&self._phantom,
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
    pub enum UpdaterMessage<T: Peb> {
        Ask(UpdaterAskMessage<T>),
        Tell(UpdaterTellMessage<T>),
    }
    #[automatically_derived]
    impl<T: ::core::fmt::Debug + Peb> ::core::fmt::Debug for UpdaterMessage<T> {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match self {
                UpdaterMessage::Ask(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "Ask",
                        &__self_0,
                    )
                }
                UpdaterMessage::Tell(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "Tell",
                        &__self_0,
                    )
                }
            }
        }
    }
    impl<T: Peb> Updater<T> {
        pub fn runs<
            S: pakka::ChannelSender<UpdaterMessage<T>>,
            R: pakka::Channel<updater_t::UpdaterMessage<T>> + Send + 'static,
        >(mut self, (tx, rx ): (S, R)) -> UpdaterHandle<T, S> {
            tokio::spawn(async move {
                let mut ctx = pakka::ActorCtx::new(rx);
                while let Ok(msg) = ctx.rx.recv().await {
                    self.handle_message(msg, &mut ctx).await;
                    if ctx.kill_flag {
                        break;
                    }
                }
                self.exit();
            });
            UpdaterHandle {
                sender: tx,
                _phantom: Default::default(),
            }
        }
        pub fn run(mut self) -> UpdaterHandle<T, Sender<UpdaterMessage<T>>> {
            let (tx, mut rx) = channel::<UpdaterMessage<T>>(100);
            self.runs(tx, rx)
        }
        fn exit(&self) {
            {
                ::std::io::_print(format_args!("{0} actor task exiting\n", "Updater"));
            };
        }
        async fn handle_message(
            &mut self,
            msg: UpdaterMessage<T>,
            mut _ctx: &mut pakka::ActorCtx<
                impl pakka::Channel<updater_t::UpdaterMessage<T>>,
                UpdaterMessage<T>,
            >,
        ) {
            match msg {
                UpdaterMessage::Ask(ask_msg) => {
                    self.handle_asks(ask_msg, &mut _ctx).await
                }
                UpdaterMessage::Tell(tell_msg) => {
                    self.handle_tells(tell_msg, &mut _ctx).await
                }
            }
        }
        async fn handle_asks(
            &mut self,
            msg: UpdaterAskMessage<T>,
            mut _ctx: &mut pakka::ActorCtx<
                impl pakka::Channel<updater_t::UpdaterMessage<T>>,
                UpdaterMessage<T>,
            >,
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
            mut _ctx: &mut pakka::ActorCtx<
                impl pakka::Channel<updater_t::UpdaterMessage<T>>,
                UpdaterMessage<T>,
            >,
        ) {
            match msg {
                UpdaterTellMessage::__Phantom(_) => {}
            }
        }
    }
    impl<T: Peb, S> UpdaterHandle<T, S>
    where
        S: pakka::ChannelSender<UpdaterMessage<T>>,
    {
        pub fn new(sender: S) -> Self {
            Self {
                sender,
                _phantom: Default::default(),
            }
        }
        pub async fn update(&self) -> Result<(u32), pakka::ActorError> {
            let (tx, rx) = tokio::sync::oneshot::channel();
            self.sender.send(UpdaterMessage::Ask(UpdaterAskMessage::Update(tx))).await?;
            rx.await.map_err(Into::into)
        }
    }
}
pub use updater_t::*;
#[allow(dead_code)]
impl<T: Peb> Updater<T> {
    fn update(
        &mut self,
        _ctx: &mut pakka::ActorCtx<
            impl pakka::Channel<updater_t::UpdaterMessage<T>>,
            updater_t::UpdaterMessage<T>,
        >,
    ) -> u32 {
        self.peb.update()
    }
}
fn main() {
    let body = async {
        let updater = Updater { peb: Test { frame: 0 } }.run();
        let frame = updater.update().await.unwrap();
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
