use std::{collections::{hash_map::Entry, HashMap}, net::SocketAddr, sync::Arc, time::Duration};

use pakka::{actor, messages};
use tokio::net::UdpSocket;

#[actor]
pub struct ChatUser {
    addr: SocketAddr,
    sock: Arc<UdpSocket>,
    name: String,
    chat_handle: ChatHandle,

}

#[messages]
impl ChatUser {
    async fn chat_send(&self, msg: String) {
        _ = self.sock.send_to(msg.as_bytes(), self.addr).await;
    }

    async fn client_send(&self, msg: String) {
        let msg = format!("{}: {}", self.name, msg);
        self.chat_handle.send_message(msg).await;
    }
}

#[derive(Default)]
#[actor]
pub struct Chat {
    pub count: u32,
    pub users: Vec<ChatUserHandle>,
}

impl Chat {
    async fn broadcast(&self, msg: String) {
        for user in &self.users {
            user.chat_send(msg.clone()).await;
        }
    }
}

#[messages]
impl Chat {
    pub fn new() -> Self {
        Chat::default()
    }

    async fn join(&mut self, name: String, addr: SocketAddr, sock: Arc<UdpSocket>, chat_handle: ChatHandle) -> ChatUserHandle {
        self.broadcast(format!("{} joined", name)).await;
        let user_handle = ChatUser {name, addr, sock, chat_handle}.run();
        self.users.push(user_handle.clone());
        user_handle
    }

    async fn send_message(&mut self, msg: String) {
        self.broadcast(msg).await
    }
}

#[tokio::main]
async fn main()  {

    let server_addr: SocketAddr = "0.0.0.0:8080".parse().unwrap();
    tokio::spawn(run_server(server_addr));

    for i in 0..10 {
        tokio::time::sleep(Duration::from_millis(666)).await;
        tokio::spawn(client(format!("client({i})"), server_addr));
    }

    tokio::time::sleep(Duration::from_secs(100)).await;
}

async fn run_server(server_addr: SocketAddr) {
    let sock = Arc::new(UdpSocket::bind(server_addr).await.unwrap());
    let chat = Chat::new().run();
    let mut users: HashMap<SocketAddr, ChatUserHandle> = Default::default();

    let mut buf = [0; 1024];

    // With Arc<DashMap> / Rw<HashMap> we could have multiple tasks receiving client messages
    loop {
        match sock.recv_from(&mut buf).await {
            Ok((len, addr)) => { 
                match std::str::from_utf8(&buf[..len]) {
                    Ok(msg) => {
                        if let Entry::Vacant(entry) = users.entry(addr) {
                            //let first message be the name
                            let handle = chat.join(msg.into(), addr, sock.clone(), chat.clone()).await;
                            entry.insert(handle);
                        } else {
                            //rest of the messages are sent to chat
                            let player = users.get_mut(&addr).unwrap();
                            player.client_send(msg.into()).await;
                        };
                    },
                    Err(_) => println!("Server received INVALID data"),
                }
            }
            Err(err) => panic!("Error: {}", err),
        }
    }
}

async fn client(name: String, server_addr: SocketAddr) {
    let sock = UdpSocket::bind("0.0.0.0:0").await.unwrap();
    sock.send_to(name.as_bytes(), server_addr).await.unwrap();
    tokio::time::sleep(Duration::from_secs(1)).await;

    let mut interval = tokio::time::interval(Duration::from_secs(2));
    let mut msg_counter = 0;
    loop {
        tokio::select! {
            msg = recv(&sock) => {
                println!("{} received: {}", name, msg)
            },
            _ = interval.tick() => {
                sock.send_to(generate_message(msg_counter).as_bytes(), server_addr).await.unwrap();
                msg_counter += 1;
            }
        }
    }
}

async fn recv(sock: &UdpSocket) -> String {
    let mut buf = [0; 1024];
    match sock.recv_from(&mut buf).await {
        Ok((len, _)) => {
            match std::str::from_utf8(&buf[..len]) {
                Ok(str) => str.into(),
                Err(err) => panic!("Failed to deserialize ServerMessage: {err}"),
            }
        }
        Err(e) => {
            panic!("Error receiving from socket: {:?}", e);
        },
    }
}

fn generate_message(order_number: usize) -> String {
    let msg = ["Hi", "How's your day", "Goodbye, see you later"];
    msg[order_number % msg.len()].to_string()
}