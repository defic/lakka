use pakka::{messages, Actor};

pub struct Test(u16);

#[messages]
impl Test {
    fn set_value(&mut self, val: u16) {
        self.0 = val;
    }

    fn get_value(&self) -> u16 {
        self.0
    }
}

pub struct Wrapper {
    handle: TestHandleUnbounded,
}

impl Drop for Wrapper {
    fn drop(&mut self) {
        self.handle.set_value(0).unwrap();
    }
}

#[tokio::main]
async fn main() {
    let handle = Test(32).run_unbounded(vec![]);
    println!("{}", handle.get_value().await.unwrap());
    handle.set_value(16).unwrap();
    println!("{}", handle.get_value().await.unwrap());

    let wrap = Wrapper {
        handle: handle.clone(),
    };
    drop(wrap);
    println!("{}", handle.get_value().await.unwrap());
}
