#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
use std::time::Duration;
use pakka::{messages, Actor};
pub struct Connected;
#[automatically_derived]
impl ::core::fmt::Debug for Connected {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::write_str(f, "Connected")
    }
}
#[automatically_derived]
impl ::core::clone::Clone for Connected {
    #[inline]
    fn clone(&self) -> Connected {
        Connected
    }
}
pub struct Happy;
pub struct Disconnected;
#[automatically_derived]
impl ::core::fmt::Debug for Disconnected {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::write_str(f, "Disconnected")
    }
}
#[automatically_derived]
impl ::core::clone::Clone for Disconnected {
    #[inline]
    fn clone(&self) -> Disconnected {
        Disconnected
    }
}
pub struct Connection<State> {
    _state: State,
}
#[automatically_derived]
impl<State: ::core::fmt::Debug> ::core::fmt::Debug for Connection<State> {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field1_finish(
            f,
            "Connection",
            "_state",
            &&self._state,
        )
    }
}
use std::marker::PhantomData;
impl pakka::Actor for Connection<Connected> {
    type Ask = ConnectionAskMessage<Connected>;
    type Tell = ConnectionTellMessage<Connected>;
    type Handle = ConnectionHandle<Connected>;
    async fn handle_asks(
        &mut self,
        msg: Self::Ask,
        mut _ctx: &mut pakka::ActorContext<Self>,
    ) {
        match msg {
            ConnectionAskMessage::Disconnect(resp) => {
                let result = self.disconnect(&mut _ctx);
                let _ = resp.send(result);
            }
            ConnectionAskMessage::_Phantom(_) => {}
        }
    }
    async fn handle_tells(
        &mut self,
        msg: Self::Tell,
        mut _ctx: &mut pakka::ActorContext<Self>,
    ) {
        match msg {
            ConnectionTellMessage::_Phantom(_) => {}
        }
    }
}
pub struct ConnectionHandle<Connected> {
    sender: Box<dyn pakka::ChannelSender<ConnectionMessage>>,
    _phantom: (std::marker::PhantomData<Connected>),
}
#[automatically_derived]
impl<Connected: ::core::clone::Clone> ::core::clone::Clone
for ConnectionHandle<Connected> {
    #[inline]
    fn clone(&self) -> ConnectionHandle<Connected> {
        ConnectionHandle {
            sender: ::core::clone::Clone::clone(&self.sender),
            _phantom: ::core::clone::Clone::clone(&self._phantom),
        }
    }
}
#[automatically_derived]
impl<Connected: ::core::fmt::Debug> ::core::fmt::Debug for ConnectionHandle<Connected> {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field2_finish(
            f,
            "ConnectionHandle",
            "sender",
            &self.sender,
            "_phantom",
            &&self._phantom,
        )
    }
}
impl ConnectionHandle<Connected> {
    pub async fn disconnect(
        &self,
    ) -> Result<(ConnectionHandle<Disconnected>), pakka::ActorError> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.sender
            .send(pakka::Message::Ask(ConnectionAskMessage::Disconnect(tx)))
            .await?;
        rx.await.map_err(Into::into)
    }
}
impl pakka::ActorHandle<ConnectionMessage> for ConnectionHandle<Connected> {
    fn new(tx: Box<dyn pakka::ChannelSender<ConnectionMessage>>) -> Self {
        Self {
            sender: tx,
            _phantom: Default::default(),
        }
    }
}
pub enum ConnectionAskMessage<Connected> {
    Disconnect(tokio::sync::oneshot::Sender<ConnectionHandle<Disconnected>>),
    _Phantom(std::marker::PhantomData<Connected>),
}
#[automatically_derived]
impl<Connected: ::core::fmt::Debug> ::core::fmt::Debug
for ConnectionAskMessage<Connected> {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match self {
            ConnectionAskMessage::Disconnect(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(
                    f,
                    "Disconnect",
                    &__self_0,
                )
            }
            ConnectionAskMessage::_Phantom(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(
                    f,
                    "_Phantom",
                    &__self_0,
                )
            }
        }
    }
}
pub enum ConnectionTellMessage<Connected> {
    _Phantom(std::marker::PhantomData<Connected>),
}
#[automatically_derived]
impl<Connected: ::core::fmt::Debug> ::core::fmt::Debug
for ConnectionTellMessage<Connected> {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match self {
            ConnectionTellMessage::_Phantom(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(
                    f,
                    "_Phantom",
                    &__self_0,
                )
            }
        }
    }
}
#[automatically_derived]
impl<Connected: ::core::clone::Clone> ::core::clone::Clone
for ConnectionTellMessage<Connected> {
    #[inline]
    fn clone(&self) -> ConnectionTellMessage<Connected> {
        match self {
            ConnectionTellMessage::_Phantom(__self_0) => {
                ConnectionTellMessage::_Phantom(::core::clone::Clone::clone(__self_0))
            }
        }
    }
}
type ConnectionMessage = pakka::Message<
    <Connection<Connected> as pakka::Actor>::Ask,
    <Connection<Connected> as pakka::Actor>::Tell,
>;
#[allow(dead_code)]
impl Connection<Connected> {
    pub fn disconnect(
        &mut self,
        _ctx: &mut pakka::ActorContext<Self>,
    ) -> ConnectionHandle<Disconnected> {
        Connection { _state: Disconnected }.run()
    }
}
use std::marker::PhantomData;
impl pakka::Actor for Connection<Disconnected> {
    type Ask = ConnectionAskMessage<Disconnected>;
    type Tell = ConnectionTellMessage<Disconnected>;
    type Handle = ConnectionHandle<Disconnected>;
    async fn handle_asks(
        &mut self,
        msg: Self::Ask,
        mut _ctx: &mut pakka::ActorContext<Self>,
    ) {
        match msg {
            ConnectionAskMessage::Connect(resp) => {
                let result = self.connect(&mut _ctx);
                let _ = resp.send(result);
            }
            ConnectionAskMessage::_Phantom(_) => {}
        }
    }
    async fn handle_tells(
        &mut self,
        msg: Self::Tell,
        mut _ctx: &mut pakka::ActorContext<Self>,
    ) {
        match msg {
            ConnectionTellMessage::_Phantom(_) => {}
        }
    }
}
pub struct ConnectionHandle<Disconnected> {
    sender: Box<dyn pakka::ChannelSender<ConnectionMessage>>,
    _phantom: (std::marker::PhantomData<Disconnected>),
}
#[automatically_derived]
impl<Disconnected: ::core::clone::Clone> ::core::clone::Clone
for ConnectionHandle<Disconnected> {
    #[inline]
    fn clone(&self) -> ConnectionHandle<Disconnected> {
        ConnectionHandle {
            sender: ::core::clone::Clone::clone(&self.sender),
            _phantom: ::core::clone::Clone::clone(&self._phantom),
        }
    }
}
#[automatically_derived]
impl<Disconnected: ::core::fmt::Debug> ::core::fmt::Debug
for ConnectionHandle<Disconnected> {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field2_finish(
            f,
            "ConnectionHandle",
            "sender",
            &self.sender,
            "_phantom",
            &&self._phantom,
        )
    }
}
impl ConnectionHandle<Disconnected> {
    pub async fn connect(
        &self,
    ) -> Result<(ConnectionHandle<Connected>), pakka::ActorError> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.sender.send(pakka::Message::Ask(ConnectionAskMessage::Connect(tx))).await?;
        rx.await.map_err(Into::into)
    }
}
impl pakka::ActorHandle<ConnectionMessage> for ConnectionHandle<Disconnected> {
    fn new(tx: Box<dyn pakka::ChannelSender<ConnectionMessage>>) -> Self {
        Self {
            sender: tx,
            _phantom: Default::default(),
        }
    }
}
pub enum ConnectionAskMessage<Disconnected> {
    Connect(tokio::sync::oneshot::Sender<ConnectionHandle<Connected>>),
    _Phantom(std::marker::PhantomData<Disconnected>),
}
#[automatically_derived]
impl<Disconnected: ::core::fmt::Debug> ::core::fmt::Debug
for ConnectionAskMessage<Disconnected> {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match self {
            ConnectionAskMessage::Connect(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(
                    f,
                    "Connect",
                    &__self_0,
                )
            }
            ConnectionAskMessage::_Phantom(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(
                    f,
                    "_Phantom",
                    &__self_0,
                )
            }
        }
    }
}
pub enum ConnectionTellMessage<Disconnected> {
    _Phantom(std::marker::PhantomData<Disconnected>),
}
#[automatically_derived]
impl<Disconnected: ::core::fmt::Debug> ::core::fmt::Debug
for ConnectionTellMessage<Disconnected> {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match self {
            ConnectionTellMessage::_Phantom(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(
                    f,
                    "_Phantom",
                    &__self_0,
                )
            }
        }
    }
}
#[automatically_derived]
impl<Disconnected: ::core::clone::Clone> ::core::clone::Clone
for ConnectionTellMessage<Disconnected> {
    #[inline]
    fn clone(&self) -> ConnectionTellMessage<Disconnected> {
        match self {
            ConnectionTellMessage::_Phantom(__self_0) => {
                ConnectionTellMessage::_Phantom(::core::clone::Clone::clone(__self_0))
            }
        }
    }
}
type ConnectionMessage = pakka::Message<
    <Connection<Disconnected> as pakka::Actor>::Ask,
    <Connection<Disconnected> as pakka::Actor>::Tell,
>;
#[allow(dead_code)]
impl Connection<Disconnected> {
    pub fn connect(
        &self,
        _ctx: &mut pakka::ActorContext<Self>,
    ) -> ConnectionHandle<Connected> {
        Connection { _state: Connected }.run()
    }
}
fn main() {
    let body = async {
        let con = Connection {
            _state: Connected,
            _feel: Happy,
        };
        let handle = con.run();
        let d = handle.disconnect().await.unwrap();
        {
            ::std::io::_print(format_args!("received disconnect\n"));
        };
        drop(handle);
        let _k = d.connect().await;
        {
            ::std::io::_print(format_args!("received connected\n"));
        };
        tokio::time::sleep(Duration::from_millis(50)).await;
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
