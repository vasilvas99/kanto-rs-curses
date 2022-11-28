use cursive::align::HAlign;
use cursive::views::Dialog;
use cursive::{traits::*, Cursive};
use cursive_table_view::TableView;
use kantocurses::kanto_api;
use std::cmp::Ordering;
use tokio::sync::mpsc;
use nix::unistd::Uid;

pub mod containers_table_view;
use containers_table_view::*;

#[derive(Debug)]
enum KantoRequest {
    ListContainers,
    _CreateContainer(String, String), // Name, Registry
    StartContainer(String),           // Name
    StopContainer(String, i64),       // Name, timeout
    RemoveContainer(String),          // Name
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
                KantoRequest::_CreateContainer(name, registry) => {
                    kanto_api::create_container(&mut c, &name, &registry).await?;
                }
                KantoRequest::StartContainer(name) => {
                    kanto_api::start_container(&mut c, &name).await; // add error handling
                }
                KantoRequest::StopContainer(name, timeout) => {
                    kanto_api::stop_container(&mut c, &name, timeout).await; // add error handling
                }
                KantoRequest::RemoveContainer(name) => {
                    kanto_api::remove_container(&mut c, &name, true).await; // add error handling
                }
            }
        }
    }
}

fn get_current_container(s: &mut Cursive) -> Option<ContainersTable> {
    let t = s
        .find_name::<TableView<ContainersTable, ContainerColumn>>("table")
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
    let table = TableView::<ContainersTable, ContainerColumn>::new()
        .column(ContainerColumn::ID, "ID", |c| c.width_percent(20))
        .column(ContainerColumn::Name, "Name", |c| c.align(HAlign::Center))
        .column(ContainerColumn::Image, "Image", |c| {
            c.ordering(Ordering::Greater)
                .align(HAlign::Right)
                .width_percent(20)
        })
        .column(ContainerColumn::Running, "Running", |c| {
            c.align(HAlign::Center)
        });

    siv.add_layer(
        Dialog::around(table.with_name("table").min_size((100, 150)))
            .title("Kanto-CM curses")
            .button("Create", |_s| { todo!() })
            .button("Start",  glib::clone!(@strong tx_requests => move |s| {
                if let Some(c) = get_current_container(s) {
                    tx_requests.blocking_send(KantoRequest::StartContainer(c.name.clone())).expect("IO thread dead");
                }
            }))
            .button("Stop", glib::clone!(@strong tx_requests => move |s| {
                if let Some(c) = get_current_container(s) {
                    tx_requests.blocking_send(KantoRequest::StopContainer(c.name.clone(), 5)).expect("IO thread dead");
                }
            }))
            .button("Remove", glib::clone!(@strong tx_requests => move |s| {
                if let Some(c) = get_current_container(s) {
                    tx_requests.blocking_send(KantoRequest::RemoveContainer(c.name.clone())).expect("IO thread dead");
                }
            }))
    );

    siv.set_fps(5);

    // Do a similar cleanup to buttons
    siv.add_global_callback(cursive::event::Event::Refresh, move |s| {
        tx_requests
            .blocking_send(KantoRequest::ListContainers)
            .expect("Could not send");
        match rx_containers.try_recv() {
            Ok(val) => {
                let mut t = s
                    .find_name::<TableView<ContainersTable, ContainerColumn>>("table")
                    .expect("Crap");
                let last_item = t.item(); // Cache the position of the table selector
                t.set_items(items_to_columns(val));
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
