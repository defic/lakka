
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
use std::fmt;
use pakka::Actor;
use pakka::messages;

#[derive(Debug, Clone)]
pub struct Test {
    frame: u32,
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
pub struct Updater<T: Peb + fmt::Debug> {
    peb: T,
}
mod updater_t {
    use std::marker::PhantomData;
    use super::*;
    impl<T: Peb + fmt::Debug> pakka::Actor for Updater<T> {
        type Ask = UpdaterAskMessage<T>;
        type Tell = UpdaterTellMessage<T>;
        type Handle = UpdaterHandle<T>;
        async fn handle_asks(
            &mut self,
            msg: Self::Ask,
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
            msg: Self::Tell,
            mut _ctx: &mut pakka::ActorContext<Self>,
        ) {
            match msg {
                UpdaterTellMessage::__Phantom(_) => {}
            }
        }
    }

    #[derive(Debug)]
    pub struct UpdaterHandle<T: Peb + fmt::Debug> {
        sender: Box<dyn pakka::ChannelSender<Message<T>>>,
    }


    impl<T: Peb + fmt::Debug> pakka::ActorHandle<Message<T>> for UpdaterHandle<T> {
        fn new(tx: Box<dyn pakka::ChannelSender<Message<T>>>) -> Self {
            Self { sender: tx }
        }
    }

    #[derive(Debug)]
    pub enum UpdaterAskMessage<T: Peb + fmt::Debug> {
        Update(tokio::sync::oneshot::Sender<u32>),
        #[doc(hidden)]
        __Phantom(PhantomData<T>),
    }

    #[derive(Debug, Clone)]
    pub enum UpdaterTellMessage<T: Peb + fmt::Debug> {
        #[doc(hidden)]
        __Phantom(PhantomData<T>),
    }

    type Message<T: Peb + fmt::Debug> = pakka::Message<
        <Updater<T> as pakka::Actor>::Ask,
        <Updater<T> as pakka::Actor>::Tell,
    >;
    impl<T: Peb + fmt::Debug> Updater<T> {
        fn exit(&self) {
            
        }
    }
    impl<T: Peb + fmt::Debug> UpdaterHandle<T> {
        pub async fn update(&self) -> Result<(u32), pakka::ActorError> {
            let (tx, rx) = tokio::sync::oneshot::channel();
            self.sender.send(pakka::Message::Ask(UpdaterAskMessage::Update(tx))).await?;
            rx.await.map_err(Into::into)
        }
    }
}
pub use updater_t::*;
#[allow(dead_code)]
impl<T: Peb + fmt::Debug> Updater<T> {
    fn update(&mut self, _ctx: &mut pakka::ActorContext<Self>) -> u32 {
        self.peb.update()
    }
}
fn main() {
    let body = async {
        let updater = Updater { peb: Test { frame: 0 } }.run();
        let frame = updater.update().await.unwrap();
        
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
