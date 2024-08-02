use std::{collections::{hash_map::Entry, HashMap}, net::SocketAddr, sync::Arc, time::{Duration, Instant}};


use pakka::{messages, Actor};

use serde::{Deserialize, Serialize};
use tokio::net::UdpSocket;

mod approx_instant {
    use std::time::{Instant, SystemTime};
    use serde::{Serialize, Serializer, Deserialize, Deserializer, de::Error};

    pub fn serialize<S>(instant: &Instant, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let system_now = SystemTime::now();
        let instant_now = Instant::now();
        let approx = system_now - (instant_now - *instant);
        approx.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Instant, D::Error>
    where
        D: Deserializer<'de>,
    {
        let de = SystemTime::deserialize(deserializer)?;
        let system_now = SystemTime::now();
        let instant_now = Instant::now();
        let duration = system_now.duration_since(de).map_err(Error::custom)?;
        let approx = instant_now - duration;
        Ok(approx)
    }
}

pub struct ChatUser {
    addr: SocketAddr,
    sock: Arc<UdpSocket>,
    name: String,
    chat_handle: ChatHandle,

}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    msg: String,
    #[serde(with = "approx_instant")]
    ts: std::time::Instant,
}

#[messages]
impl ChatUser{
    async fn chat_send(&self, msg: Message) {
        let msgbytes = bincode::serialize(&msg).unwrap();
        _ = self.sock.send_to(&msgbytes[..], self.addr).await;
    }

    async fn chat_send_bytes(&self, bytes: Arc<Vec<u8>>) {
        _ = self.sock.send_to(&bytes[..], self.addr).await;
    }

    async fn client_send(&self, msg: Message) {
        _ = self.chat_handle.send_message(msg).await;
    }
}


pub struct Chat {
    users: Vec<ChatUserHandle>,
    broadcast_sender: tokio::sync::broadcast::Sender<ChatUserTellMessage>,
}

impl Default for Chat {
    fn default() -> Self {
        let (broadcast_sender, _) = tokio::sync::broadcast::channel(100);
        Self {
            broadcast_sender,
            users: Vec::default(),
        }
    }
}

impl Chat {
    async fn broadcast(&self, msg: Message) {    
        let d = Instant::now();
        let _ = self.broadcast_sender.send(ChatUserTellMessage::ChatSend(msg));

        println!("Sending message to {} took {:?}", self.users.len(), d.elapsed());
    }

    #[allow(dead_code)]
    async fn broadcast_individually(&self, msg: Message) {    
        let d = Instant::now();
        let msg = bincode::serialize(&msg).unwrap();
        let msg = Arc::new(msg);
        for user in &self.users {
            //_ = user.chat_send(msg.clone()).await;
            _ = user.chat_send_bytes(msg.clone()).await;
        }

        println!("Sending (individually) message to {} took {:?}", self.users.len(), d.elapsed());
    }
}

#[messages]
impl Chat {

    async fn join(&mut self, name: String, addr: SocketAddr, sock: Arc<UdpSocket>, chat_handle: ChatHandle) -> ChatUserHandle {
        self.broadcast(Message {msg: "Joined".into(), ts: std::time::Instant::now()}).await;
        let ch = Box::new(self.broadcast_sender.subscribe());
        let user_handle = ChatUser {name, addr, sock, chat_handle}.run_with_channels(vec![ch]);
        self.users.push(user_handle.clone());
        user_handle
    }

    async fn send_message(&mut self, msg: Message) {
        self.broadcast(msg).await
    }
}

#[tokio::main]
async fn main()  {

    let server_addr: SocketAddr = "0.0.0.0:8080".parse().unwrap();
    tokio::spawn(run_server(server_addr));

    for i in 0..128 {
        tokio::time::sleep(Duration::from_millis(10)).await;
        tokio::spawn(client(i, format!("client({i})"), server_addr));
    }

    tokio::time::sleep(Duration::from_secs(100)).await;
}

async fn run_server(server_addr: SocketAddr) {
    let sock = Arc::new(UdpSocket::bind(server_addr).await.unwrap());
    let chat = Chat::default().run();
    let mut users: HashMap<SocketAddr, ChatUserHandle> = Default::default();

    let mut buf = [0; 1024];

    // With Arc<DashMap> / Rw<HashMap> we could have multiple tasks receiving client messages
    loop {
        match sock.recv_from(&mut buf).await {
            Ok((len, addr)) => { 
                match bincode::deserialize::<Message>(&buf[..len]) {
                //match std::str::from_utf8(&buf[..len]) {
                    Ok(msg) => {
                        if let Entry::Vacant(entry) = users.entry(addr) {
                            //let first message be the name

                            let handle = chat.join(msg.msg, addr, sock.clone(), chat.clone()).await.unwrap();
                            entry.insert(handle);
                        } else {
                            //rest of the messages are sent to chat
                            let player = users.get_mut(&addr).unwrap();

                            player.client_send(msg).await.unwrap();
                        };
                    },
                    Err(_) => println!("Server received INVALID data"),
                }
            }
            Err(err) => panic!("Error: {}", err),
        }
    }
}

async fn client(index: i32, name: String, server_addr: SocketAddr) {
    let sock = UdpSocket::bind("0.0.0.0:0").await.unwrap();
    let msg = Message {msg: name.clone(), ts: std::time::Instant::now()};
    let msg = bincode::serialize(&msg).unwrap();
    sock.send_to(&msg[..], server_addr).await.unwrap();
    tokio::time::sleep(Duration::from_secs(1)).await;

    let mut interval = tokio::time::interval(Duration::from_millis(10));
    let mut msg_counter = 0;
    loop {
        tokio::select! {
            msg = recv(&sock) => {
                println!("{} received: {}, elapsed: {:?}", name, msg.msg, msg.ts.elapsed())
            },
            _ = interval.tick() => {
                if index == 0 {
                    let msg = generate_message(msg_counter);
                    let msg = bincode::serialize(&msg).unwrap();
                    sock.send_to(&msg[..], server_addr).await.unwrap();
                    msg_counter += 1;
                }
            }
        }
    }
}

async fn recv(sock: &UdpSocket) -> Message {
    let mut buf = [0; 1024];
    match sock.recv_from(&mut buf).await {
        Ok((len, _)) => {
            match bincode::deserialize::<Message>(&buf[..len]) {
                Ok(msg) => msg,
                Err(err) => panic!("Failed to deserialize ServerMessage: {err}"),
            }
        }
        Err(e) => {
            panic!("Error receiving from socket: {:?}", e);
        },
    }
}

fn generate_message(order_number: usize) -> Message {
    let msg = ["Hi", "How's your day", "Goodbye, see you later"];
    let msg = msg[order_number % msg.len()].to_string();
    Message {msg, ts: std::time::Instant::now()}
}