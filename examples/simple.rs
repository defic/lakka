
use std::time::Duration;
use pakka::{actor, messages};

#[actor]
#[derive(Default)]
struct GameLobby {
    number: u32,
    players: Vec<String>,
}

#[messages]
impl GameLobby {

    pub fn new() -> Self {
        GameLobby::default()
    }

    fn calculation(&mut self, number: u32) -> u32{
        self.number = Self::do_calculation(self.number, number);
        self.number
    }

    fn do_calculation(n1: u32, n2: u32) -> u32 {
        n1 * n2
    }

    fn add_player(&mut self, player: String) {
        println!("Adding player: {}", player);
        self.players.push(player);
    }

    fn get_player_list(&self) -> Vec<String> {
        self.players.clone()
    }
}


#[tokio::main]
async fn main() {
    let asd = GameLobby::new();
    let handle = asd.run();
    handle.calculation(1).await;
    handle.calculation(3).await;
    handle.calculation(3).await;
    handle.calculation(3).await;

    let players = handle.get_player_list().await;
    println!("players {:?}", players);

    handle.add_player("jeps".into()).await;
    handle.add_player("jups".into()).await;
    let players = handle.get_player_list().await;
    println!("updated players: {:?}", players);

    tokio::time::sleep(Duration::from_millis(50)).await;
}
