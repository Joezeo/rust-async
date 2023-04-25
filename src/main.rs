use futures::executor::block_on;

struct Song {}

async fn learn_song() -> Song {
    println!("Learn song");
    Song {}
}

async fn sing_song(_: Song) {
    println!("Sing song");
}

async fn dance() {
    println!("Dance");
}

async fn learn_and_sing_song() {
    let song = learn_song().await;
    sing_song(song).await;
}

async fn async_main() {
    let f1 = learn_and_sing_song();
    let f2 = dance();
    futures::join!(f1, f2);
}

fn main() {
    block_on(async_main())
}
