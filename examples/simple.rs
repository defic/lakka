use lakka::*;
use std::time::Duration;

//#[actor]
#[derive(Default)]
struct SimpleTest {
    counter: u32,
    last_value: String,
}

#[messages]
impl SimpleTest {
    pub fn new() -> Self {
        SimpleTest::default()
    }

    fn last_value(&self) -> String {
        self.last_value.clone()
    }

    async fn last_value_async(&self) -> String {
        tokio::time::sleep(Duration::from_millis(15)).await;
        self.last_value.clone()
    }

    fn set_last_value(&mut self, value: String) {
        self.counter += 1;
        self.last_value = value;
    }

    async fn set_last_value_async(&mut self, value: String) {
        self.counter += 1;
        tokio::time::sleep(Duration::from_millis(15)).await;
        self.last_value = value;
    }

    fn print(&self) {
        println!(
            "actor print: {}, altered: {} times",
            self.last_value, self.counter
        );
    }
}

#[tokio::main]
async fn main() -> Result<(), ActorError> {
    //let channel = tokio::sync::mpsc::channel(100);
    let asd = SimpleTest::new().run(); //s(channel);

    asd.set_last_value("innit".into()).await?;
    asd.print().await?;
    asd.set_last_value("monkey".into()).await?;
    asd.set_last_value_async("bononoke".into()).await?;
    let res = asd.last_value().await;
    println!("Got last value: {}", res.unwrap());
    asd.set_last_value("monkey".into()).await?;
    asd.set_last_value("donkey".into()).await?;
    asd.print().await?;
    _ = asd;

    tokio::time::sleep(Duration::from_millis(50)).await;
    Ok(())
}
