use std::time::Duration;

use pakka::{messages, Interval};


pub struct Test {}

#[messages]
impl Test {
    fn ping(&self) {
        println!("Received ping")
    }
    fn test(&self) {
        println!("TEST!")
    }
}

#[tokio::main]
async fn main() {
    let pinger = Box::new(
        Interval {
            interval:tokio::time::interval(Duration::from_secs(1)),
            message: TestTellMessage::Ping()
        }
    );
    let handle = Test{}.run_with_channels(vec![pinger]);
    tokio::time::sleep(Duration::from_secs(2)).await;
    _ = handle.test().await;
    tokio::time::sleep(Duration::from_secs(60)).await;
}