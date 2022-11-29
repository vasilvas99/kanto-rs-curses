use cursive::views::Dialog;
use cursive::{traits::*, Cursive};
use cursive_table_view::TableView;
use kantocurses::{kanto_api, containers_table_view as table};
use tokio::sync::mpsc;
use nix::unistd::Uid;

#[derive(Debug)]
enum KantoRequest {
    ListContainers,
    _CreateContainer(String, String), // Name, Registry
    StartContainer(String),           // ID
    StopContainer(String, i64),       // ID, timeout
    RemoveContainer(String),          // ID
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
        if let Some(request) = request_rx.recv().await {
            match request {
                // Handle errors io thread! Otherwise the whole thing crashes a lot!
                KantoRequest::ListContainers => {
                    let r = kantocurses::kanto_api::list_containers(&mut c).await?;
                    response_tx.send(r).await?;
                }
                KantoRequest::_CreateContainer(id, registry) => {
                    kanto_api::create_container(&mut c, &id, &registry).await?;
                }
                KantoRequest::StartContainer(id) => {
                    kanto_api::start_container(&mut c, &id).await; // add err state consumption
                }
                KantoRequest::StopContainer(id, timeout) => {
                    kanto_api::stop_container(&mut c, &id, timeout).await; // add err state consumption
                }
                KantoRequest::RemoveContainer(id) => {
                    kanto_api::remove_container(&mut c, &id, true).await; // add err state consumption
                }
            }
        }
    }
}

fn get_current_container(s: &mut Cursive) -> Option<table::ContainersTable> {
    let t = s
        .find_name::<TableView<table::ContainersTable, table::ContainerColumn>>("table")
        .expect("Crap");

    if let Some(container_idx) = t.item() {
        if let Some(container) = t.borrow_item(container_idx) {
            return Some(container.clone()); // small enough struct to be worth it
        }
    }
    None
}

fn run_ui(
    tx_requests: mpsc::Sender<KantoRequest>,
    mut rx_containers: mpsc::Receiver<Vec<kanto_api::Container>>,
) -> kanto_api::Result<()> {
    
    let mut siv = cursive::default();
    
    // Split in a function
    let table = table::generate_table_view();

    let start_cb = enclose::enclose!((tx_requests) move |s: &mut Cursive| {
        if let Some(c) = get_current_container(s) {
            tx_requests.blocking_send(KantoRequest::StartContainer(c.id.clone())); // add err state consumption
        }
    });

    let stop_cb = enclose::enclose!((tx_requests) move |s: &mut Cursive| {
        if let Some(c) = get_current_container(s) {
            tx_requests.blocking_send(KantoRequest::StopContainer(c.id.clone(), 5)); // add err state consumption
        }
    });
    
    let remove_cb = enclose::enclose!((tx_requests)move |s: &mut Cursive| {
        if let Some(c) = get_current_container(s) {
            tx_requests.blocking_send(KantoRequest::RemoveContainer(c.id.clone())); // add err state consumption
        }
    });

    siv.add_layer(
        Dialog::around(table.with_name("table").min_size((100, 150)))
            .title("Kanto-CM curses")
            // .button("Create", |_s| { todo!() })
            .button("[S]tart",  start_cb.clone())
            .button("Sto[P]", stop_cb.clone())
            .button("[R]emove", remove_cb.clone())
    );

    // Add keyboard shortcuts
    siv.add_global_callback('s', start_cb.clone());
    siv.add_global_callback('p', stop_cb.clone());
    siv.add_global_callback('r', remove_cb.clone());

    siv.set_fps(5);

    // Do a similar cleanup to buttons
    siv.add_global_callback(cursive::event::Event::Refresh, move |s| {
        tx_requests
            .blocking_send(KantoRequest::ListContainers)
            .expect("Could not send");
        match rx_containers.try_recv() {
            Ok(val) => {
                let mut t = s
                    .find_name::<TableView<table::ContainersTable, table::ContainerColumn>>("table")
                    .expect("Crap");
                let last_item = t.item(); // Cache the position of the table selector
                t.set_items(table::items_to_columns(val));
                if let Some(idx) = last_item {
                    // If such a position existed, set it where it was
                    t.set_selected_item(idx);
                }
            }
            Err(_e) => {}
        }
    });
    siv.run();
    Ok(())
}
fn main() -> kanto_api::Result<()> {

    if !Uid::effective().is_root() {
        eprintln!("You must run this executable as root");
        std::process::exit(-1);
    }

    let (tx_containers, rx_containers) = mpsc::channel::<Vec<kanto_api::Container>>(32);
    let (tx_requests, mut rx_requests) = mpsc::channel::<KantoRequest>(32);
    let socket = "/run/container-management/container-management.sock";


    std::thread::spawn(move || {
        tokio_main(tx_containers, &mut rx_requests, socket).expect("Error in io thread");
    });

    run_ui(tx_requests, rx_containers)?;
    Ok(())
}
