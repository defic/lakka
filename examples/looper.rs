use std::time::Duration;

use duration_helper::DurationHelper;
use lakka::*;
use tokio::time::Instant;

///
/// If actor needs a gameloop with fixed timestep
///

struct Looper {
    next_update: tokio::time::Instant,
    last_frame: tokio::time::Instant,
}

const RATE: Duration = Duration::from_micros(16_660);
const SKIP_THRESHOLD: Duration = Duration::from_micros(2_660);

impl Looper {
    fn log_dilation(&mut self, now: Instant) {
        let dilation_micros =
            (now.duration_since(self.last_frame).as_micros() as i32) - (RATE.as_micros() as i32);
        println!("time dilation: {}", dilation_micros);
    }

    fn update(&mut self, now: Instant) {
        println!(
            "---- Received tick {:?}",
            now.duration_since(self.last_frame)
        );
        self.log_dilation(now);
        self.last_frame = now;
        //DO stuff
    }

    fn schedule_tick(&mut self, _ctx: &mut ActorContext<Self>) {
        let now = Instant::now();
        if now < self.next_update {
            if now < self.next_update - SKIP_THRESHOLD {
                _ctx.delayed_tell(
                    LooperTellMessage::Tick(),
                    (self.next_update - SKIP_THRESHOLD) - now,
                );
            } else {
                _ctx.tell(LooperTellMessage::Tick());
            }
        } else {
            //already too late, scheduling right away
            _ctx.tell(LooperTellMessage::Tick());
        }
    }
}

#[messages]
impl Looper {
    pub fn tick(&mut self) {
        let now = Instant::now();
        if self.next_update <= now {
            self.update(now);
            self.next_update += RATE;
        }
        self.schedule_tick(_ctx);
    }

    pub fn println(&self) {
        println!("❤️ ???    println");
    }
}

#[tokio::main]
async fn main() {
    let handle = Looper {
        next_update: tokio::time::Instant::now() + RATE,
        last_frame: tokio::time::Instant::now(),
    }
    .run();
    _ = handle.tick().await;
    tokio::time::sleep(1.secs()).await;
    _ = handle.println().await;
    tokio::time::sleep(100.secs()).await
}
