use std::{error::Error, time::Duration};

use lakka::*;

#[derive(Debug)]
enum State {
    Uuno,
    Zuumo(u32),
}

#[messages]
impl State {
    fn print(&self) {
        println!("test {:?}", self);
        _ctx.shut_down_actor();
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let handle = State::Uuno.run();
    _ = handle.print().await;

    let handle2 = State::Zuumo(5).run();
    handle2.print().await?;

    tokio::time::sleep(Duration::from_millis(2000)).await;

    handle2.print().await.unwrap_err();

    Ok(())
}
