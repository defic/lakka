use std::time::Duration;

use duration_helper::DurationHelper;
use pakka::{messages, Actor};


struct Looper (tokio::time::Instant);

const RATE: Duration = Duration::from_micros(8_330);
const SLEEP: Duration = Duration::from_micros(6_300);
const SKIP: Duration = Duration::from_micros(200);
const SKIP_THRESHOLD: Duration = Duration::from_micros(5_700);

impl Looper {
    fn update(&mut self) {
        println!("---- Received tick {:?}", self.0.elapsed());
        self.0 = tokio::time::Instant::now();
        
    }
}

#[messages]
impl Looper {
    pub fn tick(&mut self) {
        
        if self.0.elapsed() >= RATE {
            self.update();
            _ctx.delayed_tell(LooperTellMessage::Tick(), SLEEP);
        } else if self.0.elapsed() < SKIP_THRESHOLD {
            _ctx.delayed_tell(LooperTellMessage::Tick(), SKIP);
        } else {
            _ctx.tell(LooperTellMessage::Tick());
        }
    }

    pub fn println(&self) {
        println!("❤️ ???    println");
    }
}

#[tokio::main]
async fn main(){
    let handle = Looper(tokio::time::Instant::now()).run();
    _ = handle.tick().await;
    tokio::time::sleep(1.secs()).await;
    _ = handle.println().await;
    tokio::time::sleep(100.secs()).await
}