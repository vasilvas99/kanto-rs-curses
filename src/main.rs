use cursive::views::Dialog;
use cursive::{traits::*, Cursive};
use kantocurses::{containers_table_view as table, kanto_api, try_best};
use nix::unistd::Uid;
use tokio::sync::mpsc;

#[derive(Debug)]
enum KantoRequest {
    ListContainers,
    _CreateContainer(String, String), // Name, Registry
    StartContainer(String),           // ID
    StopContainer(String, i64),       // ID, timeout
    RemoveContainer(String),          // ID
    _GetLogs(String),                  // ID
}

#[derive(Debug)]
enum KantoResponse {
    ListContainers(Vec<kanto_api::Container>),
    _GetLogs(String),
}

#[cfg(unix)]
#[tokio::main]
async fn tokio_main(
    response_tx: mpsc::Sender<KantoResponse>,
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
                    try_best(response_tx.send(KantoResponse::ListContainers(r)).await)
                }
                KantoRequest::_CreateContainer(id, registry) => {
                    try_best(kanto_api::create_container(&mut c, &id, &registry).await);
                }
                KantoRequest::StartContainer(id) => {
                    try_best(kanto_api::start_container(&mut c, &id).await);
                }
                KantoRequest::StopContainer(id, timeout) => {
                    try_best(kanto_api::stop_container(&mut c, &id, timeout).await)
                }
                KantoRequest::RemoveContainer(id) => {
                    try_best(kanto_api::remove_container(&mut c, &id, true).await)
                },
                _ => {}
            }
        }
    }
}

fn run_ui(
    tx_requests: mpsc::Sender<KantoRequest>,
    mut rx_containers: mpsc::Receiver<KantoResponse>,
) -> kanto_api::Result<()> {
    let mut siv = cursive::default();

    // Split in a function
    let table = table::generate_table_view();

    let start_cb = enclose::enclose!((tx_requests) move |s: &mut Cursive| {
        if let Some(c) = table::get_current_container(s) {
            try_best(tx_requests.blocking_send(KantoRequest::StartContainer(c.id.clone())));
        }
    });

    let stop_cb = enclose::enclose!((tx_requests) move |s: &mut Cursive| {
        if let Some(c) = table::get_current_container(s) {
            try_best(tx_requests.blocking_send(KantoRequest::StopContainer(c.id.clone(), 5)))
        }
    });

    let remove_cb = enclose::enclose!((tx_requests)move |s: &mut Cursive| {
        if let Some(c) = table::get_current_container(s) {
            try_best(tx_requests.blocking_send(KantoRequest::RemoveContainer(c.id.clone())));
        }
    });

    siv.add_layer(
        Dialog::around(table.with_name(table::TABLE_IDENTIFIER).min_size((100, 150)))
            .title("Kanto-CM curses")
            // .button("Create", |_s| { todo!() })
            .button("[S]tart", start_cb.clone())
            .button("Sto[P]", stop_cb.clone())
            .button("[R]emove", remove_cb.clone()),
    );

    // Add keyboard shortcuts
    siv.add_global_callback('s', start_cb.clone());
    siv.add_global_callback('p', stop_cb.clone());
    siv.add_global_callback('r', remove_cb.clone());

    siv.set_fps(5);

    // Do a similar cleanup to buttons
    siv.add_global_callback(cursive::event::Event::Refresh, move |s| {
        try_best(tx_requests.blocking_send(KantoRequest::ListContainers));
        if let Some(resp) = rx_containers.blocking_recv() {
            match resp {
                KantoResponse::ListContainers(list) => table::update_table_items(s, list),
                _ => {}
            }
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

    let (tx_containers, rx_containers) = mpsc::channel::<KantoResponse>(32);
    let (tx_requests, mut rx_requests) = mpsc::channel::<KantoRequest>(32);
    let socket = "/run/container-management/container-management.sock";

    std::thread::spawn(move || {
        tokio_main(tx_containers, &mut rx_requests, socket).expect("Error in io thread");
    });

    run_ui(tx_requests, rx_containers)?;
    Ok(())
}
