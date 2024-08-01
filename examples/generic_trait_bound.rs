use std::error::Error;
use std::fmt;
use std::mem;
use std::time::Duration;
use pakka::messages;
use pakka::Actor;


pub trait Game: fmt::Debug + 'static + Send + Sync + Clone {
    type Input: Clone + Sync + Send + fmt::Debug;
    
    fn update(&mut self, inputs: Vec::<Self::Input>);
}

#[derive(Default)]
struct Lobby<T: Game> {
    game: T,
    state: SomeState<T>
}

#[derive(Debug, Default, Clone)]
pub enum SomeState<T: Game>{
    #[default]
    Id,
    Awd(u32),
    Tpt{i:u32},
    Inputs(Vec<T::Input>)
}

#[messages]
impl <T: Game> Lobby<T> {
    fn player_input(&mut self, input: T::Input) {
        println!("Received player input! {:?}", input);
        match &mut self.state {
            SomeState::Inputs(inputlist) => inputlist.push(input),
            SomeState::Id | SomeState::Awd(_) | SomeState::Tpt{..}  => println!("Not adding inputs, since state is: {:?}", self.state),
        }
    }

    fn update(&mut self) {

        let inputlist = match &mut self.state {
            SomeState::Inputs(inputlist) => mem::take(inputlist),
            _ => return,
        };
        self.game.update(inputlist)
    }

    fn alter_state(&mut self, new_state: SomeState<T>) {
        println!("new_state: {:?}", new_state);
        self.state = new_state;
    }

    fn state(&self) -> SomeState<T> {
        self.state.clone()
    }
}


#[derive(Debug, Clone, Default)]
pub struct CacaGame {
    frame_number: u32,
}

impl Game for CacaGame {
    type Input = CacaInput;

    fn update(&mut self, _: Vec::<Self::Input>) {
        self.frame_number += 1;
    }
}

#[derive(Clone, Debug)]
pub struct CacaInput {
    pub up: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let gamelobby = Lobby::<CacaGame>::default();
    let handle = gamelobby.run();

    handle.player_input(CacaInput{up: true}).await?;
    handle.alter_state(SomeState::Inputs(vec![])).await?;
    handle.player_input(CacaInput{up: true}).await?;
    
    let state = handle.state().await;
    println!("State: {:?}", state);

    tokio::time::sleep(Duration::from_millis(50)).await;
    Ok(())
}

