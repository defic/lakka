use std::time::Duration;

use duration_helper::DurationHelper;
use pakka::*;

#[derive(Debug, Clone)]
pub struct EverySecond {
    msg: String,
}
pub struct Test(tokio::time::Instant);

#[messages]
impl Test {
    fn interval_test(&mut self, msg: IntervalMessage<EverySecond>) {
        println!(
            "---- Received interval_test {:?}, count {}, msg: {:?}",
            self.0.elapsed(),
            msg.counter,
            msg.msg
        );
        self.0 = tokio::time::Instant::now();
    }

    fn test(&self) {
        println!("TEST!");
    }

    fn start_interval_test(&mut self) {
        let channel = Intervaller::new(
            1.secs(),
            TestTellMessage::IntervalTest,
            EverySecond {
                msg: "noice".into(),
            },
        );
        _ctx.add_channel(Box::new(channel));
    }
}

#[tokio::main]
async fn main() {
    let handle = Test(tokio::time::Instant::now()).run();
    _ = handle.test().await;
    tokio::time::sleep(Duration::from_secs(2)).await;
    _ = handle.start_interval_test().await;
    tokio::time::sleep(Duration::from_secs(60)).await;
}
