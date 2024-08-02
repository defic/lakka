use std::time::Duration;

use duration_helper::DurationHelper;
use pakka::{messages, Actor, ConstructedMessage, Interval, IntervalCounter};


#[derive(Debug, Clone)]
pub struct CounterMessage {
    msg: String,
    counter: u32,
    variant: fn(CounterMessage) -> TestTellMessage,
}

impl ConstructedMessage for CounterMessage {
    type Variant = fn(CounterMessage) -> TestTellMessage;
    type Output = TestTellMessage;

    fn create(&self, content: Self) -> Self::Output {
        (self.variant)(content)
    }
}


impl IntervalCounter for CounterMessage {
    fn set_counter(&mut self, counter: u32) {
        self.counter = counter;
    }
}

pub struct Test(tokio::time::Instant);

#[messages]
impl Test {
    fn counter(&mut self, msg: CounterMessage) {
        println!("---- Received ping {:?}", self.0.elapsed());
        self.0 = tokio::time::Instant::now();
    }

    fn ping(&self) {
        println!("Received ping")
    }

    fn test(&self) {
        println!("TEST!");
    }

    fn start_ticker(&self) {
        let variant = TestTellMessage::Counter;
        let content = CounterMessage{counter: 0, msg: "".into(), variant};

        let ticker = Interval::new(16.600.millis(), content);
        _ctx.add_channel(Box::new(ticker));
    }
}

#[tokio::main]
async fn main() {
    
    let handle = Test(tokio::time::Instant::now()).run();
    _ = handle.test().await;
    tokio::time::sleep(Duration::from_secs(2)).await;
    _ = handle.start_ticker().await;
    tokio::time::sleep(Duration::from_secs(60)).await;
}