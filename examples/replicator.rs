use std::time::Duration;

use pakka::{messages, Actor};
use tokio::time::Instant;



pub struct OneOff{
    state: bool,
    time: Instant,
}

#[messages]
impl OneOff {
    fn process(&mut self) {
        if !self.state {
            self.state = true;
            println!("Processing this the next time: {:?}", Instant::now());

            _ctx.tell(OneOffTellMessage::Process());
            self.time = Instant::now();
        } else {
            self.state = false;
            println!("Now processing! {:?}", self.time.elapsed());
        }
    }
}

#[tokio::main]
async fn main() {
    let prog = OneOff{state: false, time: Instant::now()}.run();
    _ = prog.process().await;
    tokio::time::sleep(Duration::from_millis(500)).await;
    _ = prog.process().await;
    tokio::time::sleep(Duration::from_millis(500)).await;
}