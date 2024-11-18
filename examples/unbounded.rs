use pakka::*;

pub struct Test(u16);

#[messages(unbounded)]
impl Test {
    fn set_value(&mut self, val: u16) {
        self.0 = val;
    }

    fn get_value(&self) -> u16 {
        self.0
    }
}

pub struct TestAsync(u16);
#[messages(default)]
impl TestAsync {
    fn set_value(&mut self, val: u16) {
        self.0 = val;
    }

    fn get_value(&self) -> u16 {
        self.0
    }
}

pub struct Wrapper {
    handle: TestHandle,
}

impl Drop for Wrapper {
    fn drop(&mut self) {
        self.handle.set_value(0).unwrap();
    }
}

#[tokio::main]
async fn main() {
    let handle = Test(32).run();
    println!("{}", handle.get_value().await.unwrap());
    handle.set_value(16).unwrap();
    println!("{}", handle.get_value().await.unwrap());

    let wrap = Wrapper {
        handle: handle.clone(),
    };
    drop(wrap);
    println!("{}", handle.get_value().await.unwrap());

    // Async:
    let handle = TestAsync(32).run();
    println!("{}", handle.get_value().await.unwrap());
    handle.set_value(16).await.unwrap();
    println!("{}", handle.get_value().await.unwrap());
}
