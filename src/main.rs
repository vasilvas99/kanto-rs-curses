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
    request_rx: &mut mpsc::Receiver<KantoRequest>,
    socket_path: &str,
) -> kanto_api::Result<()> {
    let mut c = kanto_api::get_connection(socket_path).await?;
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
            }
        }
       
    }
}

// Two threads are spawned - one for sync and one for async code. Async code interfaces with kanto-cm and sends the current state of the containers
// down the channel. The main thread is only concerned with printing the state 
// TODO: Add a second channel that sends request to the async runtime such as create, start, stop etc.
// TODO-TUI: Add buttons that send the CRUD requests to async thread and print the result (open loop system)
// fn main()  -> Result<(), Box<dyn std::error::Error>>{
//     let (tx_containers, mut rx_containers) = mpsc::channel::<Vec<kanto_api::Container>>(32);
//     let (tx_requests, mut rx_requests) = mpsc::channel::<KantoRequest>(32);
//     let socket = "/run/container-management/container-management.sock";

//     std::thread::spawn(move || {
//         tokio_main(tx_containers, &mut rx_requests, socket).expect("Error in io thread");
//     });

//     loop {
//         tx_requests.blocking_send(KantoRequest::ListContainers)?;
//         match rx_containers.try_recv() {
//             Ok(val) => println!("{:#?}", val),
//             Err(_e) => {},
//         }
//     }

// }

use cursive::{views::TextView, view::Nameable};
use rand::Rng;
use cursive_table_view::{TableView, TableViewItem};

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
enum ContainerColumn {
    ID,
    Name,
    Image,
    Running
}

impl ContainerColumn {
    fn as_str(&self) -> &str {
        match *self {
            ContainerColumn::ID => "ID",
            ContainerColumn::Name => "Name",
            ContainerColumn::Image => "Image",
            ContainerColumn::Running => "Running"
        }
    }
}

#[derive(Clone, Debug)]
struct ContainersTable {
    ID: String,
    Name: String,
    Image: String,
    Running: String
}

impl TableViewItem<ContainerColumn> for <ContainersTable> {
    fn to_column(&self, column: ContainerColumn) -> String {
        match column {
            ContainerColumn::ID => format!("{}", self.ID),
            ContainerColumn::Name => format!("{}", self.Name),
            ContainerColumn::Image => format!("{}", self.Image),
            ContainerColumn::Running => format!("{}", self.Running),

        }
    }

    fn cmp(&self, other: &Self, column: ContainerColumn) -> Ordering
    where
        Self: Sized,
    {
        match column {
            ContainerColumn::ID => self.name.cmp(&other.ID),
            ContainerColumn::Name => self.name.cmp(&other.Name),
            ContainerColumn::Image => self.name.cmp(&other.Image),
            ContainerColumn::Running => self.name.cmp(&other.Running),
        }
    }
}

fn main() {
	let mut siv = cursive::default();
    
    let mut tv = TextView::new("Hello cursive! Press <q> to quit.")
                                .with_name("something");
    
	siv.add_layer(tv);
    siv.set_fps(5);

    let mut rng = rand::thread_rng();
    
    siv.add_global_callback(cursive::event::Event::Refresh, move |s| {
        let mut text = s.find_name::<TextView>("something").unwrap();
        text.set_content(format!("{}", rng.gen::<f64>()));
    });
	siv.run();
}
