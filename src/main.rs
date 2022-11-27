use cursive::{ align::HAlign, views::StackView };
use cursive::traits::*;
use cursive::views::Dialog;
use kantocurses::kanto_api;
use tokio::sync::mpsc;
use std::cmp::Ordering;
use cursive_table_view::{ TableView, TableViewItem };

pub mod ContainersTableView;
use ContainersTableView::*;

#[derive(Debug)]
enum KantoRequest {
    ListContainers,
    CreateContainer(String, String), // Name, Registry
    StartContainer(String), // Name
    StopContainer(String, i64), // Name, timeout
    RemoveContainer(String), // Name
}

#[cfg(unix)]
#[tokio::main]
async fn tokio_main(
    response_tx: mpsc::Sender<Vec<kanto_api::Container>>,
    request_rx: &mut mpsc::Receiver<KantoRequest>,
    socket_path: &str
) -> kanto_api::Result<()> {
    let mut c = kanto_api::get_connection(socket_path).await?;
    loop {
        if let Some(request) = request_rx.recv().await {
            match request {
                KantoRequest::ListContainers => {
                    let r = kantocurses::kanto_api::list_containers(&mut c).await?;
                    response_tx.send(r).await?;
                }
                KantoRequest::CreateContainer(name, registry) => {
                    kanto_api::create_container(&mut c, &name, &registry).await?;
                }
                KantoRequest::StartContainer(name) => {
                    kanto_api::start_container(&mut c, &name).await?;
                }
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

fn run_ui(
    tx_requests: mpsc::Sender<KantoRequest>,
    mut rx_containers: mpsc::Receiver<Vec<kanto_api::Container>>
) {
    let mut siv = cursive::default();
    let table = TableView::<ContainersTable, ContainerColumn>
        ::new()
        .column(ContainerColumn::ID, "ID", |c| c.width_percent(20))
        .column(ContainerColumn::Name, "Name", |c| c.align(HAlign::Center))
        .column(ContainerColumn::Image, "Image", |c| {
            c.ordering(Ordering::Greater).align(HAlign::Right).width_percent(20)
        })
        .column(ContainerColumn::Running, "Running", |c| { c.align(HAlign::Center) });

    siv.add_layer(
        Dialog::around(table.with_name("table").min_size((100, 150)))
            .title("Kanto-CM curses")
            .button("Create", |s| { todo!() })
            .button("Start", |s| { todo!() })
            .button("Stop", |s| { todo!() })
            .button("Remove", |s| { todo!() })
    );

    siv.set_fps(3);

    siv.add_global_callback(cursive::event::Event::Refresh, move |s| {
        tx_requests.blocking_send(KantoRequest::ListContainers).expect("Could not send");
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
}
fn main() {
    let (tx_containers, mut rx_containers) = mpsc::channel::<Vec<kanto_api::Container>>(32);
    let (tx_requests, mut rx_requests) = mpsc::channel::<KantoRequest>(32);
    let socket = "/run/container-management/container-management.sock";

    std::thread::spawn(move || {
        tokio_main(tx_containers, &mut rx_requests, socket).expect("Error in io thread");
    });

    run_ui(tx_requests, rx_containers);
}