use pakka::messages;

struct Actor {

}

#[messages]
impl Actor {
    fn test(&self) {
        println!("test");
    }

    fn get(&self) -> u32 {
        15
    }
}

#[tokio::main]
async fn main(){
    let test = Actor{};
    //let handle = test.run();
    //_ = handle.test().await;
    //_ = handle.get().await;
}