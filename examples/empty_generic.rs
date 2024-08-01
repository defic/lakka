use std::fmt;

use pakka::Actor;
use pakka::messages;


#[derive(Clone, Debug)]
pub struct Test {
    frame: u32
}

impl Peb for Test {
    fn update(&mut self) -> u32 {
        self.frame += 1;
        self.frame
    }
}

pub trait Peb: Send + Sync + Clone + 'static + fmt::Debug {
    fn update(&mut self) -> u32;
}

pub struct Updater<T: Peb> {
    peb: T,
}

#[messages]
impl <T: Peb> Updater<T> {
    fn update(&mut self) -> u32 {
        self.peb.update()
    }
}


#[tokio::main]
async fn main() {
    let updater = Updater { peb: Test {frame: 0 }}.run();
    let frame = updater.update().await.unwrap();
    println!("Frame {}", frame)
}