use std::time::Duration;

use lakka::*;

struct Broadcaster {
    amount_created: u32,
    broadcast_sender: tokio::sync::broadcast::Sender<BroadcastListenerTellMessage>,
}

impl Broadcaster {
    pub fn new() -> Self {
        let (broadcast_sender, _) = tokio::sync::broadcast::channel(100);
        Self {
            broadcast_sender,
            amount_created: 0,
        }
    }
}

#[messages]
impl Broadcaster {
    fn broadcast_msg(&mut self, msg: BroadcastListenerTellMessage) {
        println!("Receiver count {}", self.broadcast_sender.receiver_count());
        _ = self.broadcast_sender.send(msg);
    }

    fn create_broadcast_receiver_actor(&mut self) -> BroadcastListenerHandle {
        self.amount_created += 1;

        let ch = Box::new(self.broadcast_sender.subscribe());

        BroadcastListener {}.run_with_channels(vec![ch])
        //BroadcastListener {}.run_with_broadcast_receiver()
    }
}

struct BroadcastListener {}

#[messages]
impl BroadcastListener {
    fn message(&self, msg: String) {
        println!("Got message: {}", msg);
    }

    fn number(&self, number: f32) {
        println!("Got number {}", number);
    }
}

#[tokio::main]
async fn main() -> Result<(), ActorError> {
    let broadcaster = Broadcaster::new().run();

    let listener_handle_1 = broadcaster.create_broadcast_receiver_actor().await?;
    let listener_handle_2 = broadcaster.create_broadcast_receiver_actor().await?;
    let listener_handle_3 = broadcaster.create_broadcast_receiver_actor().await?;

    listener_handle_3.number(16.0).await?;
    broadcaster
        .broadcast_msg(BroadcastListenerTellMessage::Message(
            "Hello from broadcaster, 3 receivers".into(),
        ))
        .await?;
    tokio::time::sleep(Duration::from_millis(100)).await;
    std::mem::drop(listener_handle_1);

    broadcaster
        .broadcast_msg(BroadcastListenerTellMessage::Message("2 receivers".into()))
        .await?;
    tokio::time::sleep(Duration::from_millis(100)).await;
    std::mem::drop(listener_handle_2);

    broadcaster
        .broadcast_msg(BroadcastListenerTellMessage::Message("1 receiver".into()))
        .await?;
    tokio::time::sleep(Duration::from_millis(100)).await;
    std::mem::drop(listener_handle_3);

    broadcaster
        .broadcast_msg(BroadcastListenerTellMessage::Message("No receivers".into()))
        .await?;
    tokio::time::sleep(Duration::from_millis(100)).await;

    Ok(())
}
