
use std::time::Duration;

use pakka::{messages, Actor};

#[derive(Debug, Clone)]
pub struct Connected;

#[derive(Debug, Clone)]
pub struct Happy;

#[derive(Debug, Clone)]
pub struct Disconnected;

#[derive(Debug)]
pub struct Connection<State, Feel> {
    _state: State,
    _feel: Feel 
}

#[messages]
impl Connection<Connected, Happy> {
     
    pub fn disconnect(&mut self) -> connection_disconnected_happy::ConnectionHandle<Disconnected, Happy> {
        Connection{_state: Disconnected, _feel: Happy}.run()
    }
}

#[messages]
impl Connection<Disconnected, Happy> {
     
    pub fn connect(&self) -> connection_connected_happy::ConnectionHandle<Connected, Happy> {
        Connection{_state: Connected, _feel: Happy}.run()
    }
}


#[tokio::main]
async fn main() {
    let con = Connection{_state: Connected, _feel: Happy};
    let handle = con.run();
    let d = handle.disconnect().await.unwrap();
    println!("received disconnect", );

    drop(handle);
    let _k = d.connect().await;
    println!("received connected");

    tokio::time::sleep(Duration::from_millis(50)).await;
}

