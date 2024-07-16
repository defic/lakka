use std::{marker::PhantomData, time::Duration};

use pakka::messages;

#[derive(Debug)]
pub struct Connected;

#[derive(Debug)]
pub struct Disconnected;

#[derive(Debug)]
pub struct Connection<State> {
    state: State
}

#[messages]
impl Connection<Connected> {

    //this doesn't make sense, but maybe there is more to this.
    pub fn disconnect(&mut self) -> Connection<Disconnected>{
        Connection{state: Disconnected}
    }
}

/* Doesn't work yet

#[messages]
impl Connection<Disconnected> {
    pub fn connect(&self) {
        println!("connecting...")
    }
}
 */


#[tokio::main]
async fn main() {
    let con = Connection{state: Connected};
    let handle = con.run();
    let d = handle.disconnect().await;
    println!("received {:?}", d);

    tokio::time::sleep(Duration::from_millis(50)).await;
}