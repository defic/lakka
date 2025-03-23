use pakka::*;
use std::time::Duration;

#[derive(Default)]
struct Counter {
    counter: i32,
}

//Only macro needed to use pakka. Will generate CounterHandle, necessary enums for messages and implement Actor trait etc.
#[messages]
impl Counter {
    // modify actor state
    fn inc(&mut self) {
        self.counter += 1;
    }

    fn get(&self) -> i32 {
        self.counter
    }
}

#[tokio::main]
async fn main() {
    // Run will run the actor in a tokio task.
    let counter_handle = Counter::default().run();
    // Functions with no return values from the actor are considered as "Actor Tell" message.
    _ = counter_handle.inc().await;
    _ = counter_handle.inc().await;
    // "Actor Ask" message when there is a return value
    let state = counter_handle.get().await.unwrap();
    assert_eq!(state, 2);

    let second_handle = counter_handle.clone();
    tokio::spawn(async move {
        loop {
            second_handle.inc().await.unwrap();
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    });

    tokio::time::sleep(Duration::from_secs(1)).await;
    println!("Counter value: {}", counter_handle.get().await.unwrap());
}
