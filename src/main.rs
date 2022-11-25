use tokio::sync::mpsc;
use kantocurses::kanto_api;

#[cfg(unix)]
#[tokio::main]
async fn tokio_main(tx: mpsc::Sender<Vec<kanto_api::Container>>) -> kanto_api::Result<()> {
    let mut c = kanto_api::get_connection("/run/container-management/container-management.sock").await?;
    loop {
        let r = kantocurses::kanto_api::list_containers(&mut c).await?;
        tx.send(r).await?;
    }
}

// Two threads are spawned - one for sync and one for async code. Async code interfaces with kanto-cm and sends the current state of the containers
// down the channel. The main thread is only concerned with printing the state 
// TODO: Add a second channel that sends request to the async runtime such as create, start, stop etc.
// TODO-TUI: Add buttons that send the CRUD requests to async thread and print the result (open loop system)
fn main() {
    let (tx, mut rx) = mpsc::channel::<Vec<kanto_api::Container>>(32);

    std::thread::spawn(move || {
        tokio_main(tx).unwrap();
    });

    loop {
        match rx.try_recv() {
            Ok(val) => println!("{:#?}", val),
            Err(_e) => println!("No data received")
        }
    }
}
