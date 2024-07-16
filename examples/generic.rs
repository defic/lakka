use std::time::Duration;

use pakka::messages;

#[derive(Debug)]
pub struct Connected;

#[derive(Debug)]
pub struct Disconnected;

#[derive(Debug)]
pub struct Connection<State> {
    _state: State
}

#[messages]
impl Connection<Connected> {

    pub fn disconnect(&mut self) -> connection_disconnected::ConnectionHandle{
        Connection{_state: Disconnected}.run()
    }
}

#[messages]
impl Connection<Disconnected> {
    pub fn connect(&self) -> connection_connected::ConnectionHandle {
        Connection{_state: Connected}.run()
    }
}


#[tokio::main]
async fn main() {
    let con = Connection{_state: Connected};
    let handle = con.run();
    let d = handle.disconnect().await;
    println!("received disconnect", );

    drop(handle);
    let _k = d.connect().await;
    println!("received connected");

    tokio::time::sleep(Duration::from_millis(50)).await;
}