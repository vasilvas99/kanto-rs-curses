use tokio::sync::mpsc;
use kantocurses::kanto_api;

#[derive(Debug)]
enum KantoRequest {
    ListContainers,
    CreateContainer(String, String), // Name, Registry
    StartContainer(String), // Name
    StopContainer(String, i64), // Name, timeout
    RemoveContainer(String) // Name
}

#[cfg(unix)]
#[tokio::main]
async fn tokio_main(
    response_tx: mpsc::Sender<Vec<kanto_api::Container>>, 
    request_rx: &mut mpsc::Receiver<KantoRequest>
) -> kanto_api::Result<()> {
    let mut c = kanto_api::get_connection("/run/container-management/container-management.sock").await?;
    loop {
        if let Some(request)  = request_rx.recv().await {
            match request {
                KantoRequest::ListContainers => {
                    let r = kantocurses::kanto_api::list_containers(&mut c).await?;
                    response_tx.send(r).await?;
                },
                KantoRequest::CreateContainer(name,registry) => {
                    kanto_api::create_container(&mut c, &name, &registry).await?;
                },
                KantoRequest::StartContainer(name) => {
                    kanto_api::start_container(&mut c, &name).await?;
                },
                KantoRequest::StopContainer(name, timeout) => {
                    kanto_api::stop_container(&mut c, &name, timeout).await?;
                }
                KantoRequest::RemoveContainer(name) => {
                    kanto_api::remove_container(&mut c, &name, true).await?;
                }
                _ => println!("Unsupported Request")
            }
        }
       
    }
}

// Two threads are spawned - one for sync and one for async code. Async code interfaces with kanto-cm and sends the current state of the containers
// down the channel. The main thread is only concerned with printing the state 
// TODO: Add a second channel that sends request to the async runtime such as create, start, stop etc.
// TODO-TUI: Add buttons that send the CRUD requests to async thread and print the result (open loop system)
fn main()  -> Result<(), Box<dyn std::error::Error>>{
    let (tx_containers, mut rx_containers) = mpsc::channel::<Vec<kanto_api::Container>>(32);
    let (tx_requests, mut rx_requests) = mpsc::channel::<KantoRequest>(32);
    
    std::thread::spawn(move || {
        tokio_main(tx_containers, &mut rx_requests).expect("Error in io thread");
    });

    loop {
        tx_requests.blocking_send(KantoRequest::ListContainers)?;
        match rx_containers.try_recv() {
            Ok(val) => println!("{:#?}", val),
            Err(_e) => {},
        }
    }

}
