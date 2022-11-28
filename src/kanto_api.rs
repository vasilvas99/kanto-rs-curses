#[cfg(unix)]
use tokio::net::UnixStream;
use tonic::transport::{Endpoint, Uri};
use tower::service_fn;
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
pub type ClientChannel = cm_rpc::containers_client::ContainersClient<tonic::transport::Channel>;

mod containers {
    //This is a hack because tonic has an issue with deeply nested protobufs
    tonic::include_proto!("mod");
}
pub use containers::github::com::eclipse_kanto::container_management::containerm::api::services::containers::{self as cm_rpc, CreateContainerResponse};
pub use containers::github::com::eclipse_kanto::container_management::containerm::api::types::containers::{self as cm_types, Container};

const CONT_TEMPLATE: &'static str = include_str!("container_json_template.in");

pub async fn get_connection(socket_path: &str) -> Result<ClientChannel> {
    let _path = socket_path.to_owned();
    let channel = Endpoint::try_from("http://[::]:50051")?
        .connect_with_connector(service_fn(move |_: Uri| UnixStream::connect(_path.clone())))
        .await?;
    Ok(cm_rpc::containers_client::ContainersClient::new(channel))
}

pub async fn list_containers(channel: &mut ClientChannel) -> Result<Vec<cm_types::Container>> {
    let _r = tonic::Request::new(cm_rpc::ListContainersRequest {});
    let containers = channel.list(_r).await?.into_inner();
    Ok(containers.containers)
}

pub async fn create_container(
    channel: &mut ClientChannel,
    name: &str,
    registry: &str,
) -> Result<CreateContainerResponse> {
    let mut template: Container = serde_json::from_str(CONT_TEMPLATE)?;
    template.name = String::from(name);
    template.image.as_mut().ok_or("Field name missing")?.name = String::from(registry);

    let _r = tonic::Request::new(cm_rpc::CreateContainerRequest {
        container: Some(template),
    });
    let _response = channel.create(_r).await?;
    Ok(_response.into_inner())
}

pub async fn get_container_by_name(channel: &mut ClientChannel, name: &str) -> Result<Container> {
    let all_containers = list_containers(channel).await?;
    eprintln!("{:#?}", name);
    let cont = all_containers
        .into_iter()
        .find(|c| c.name == String::from(name))
        .ok_or("Container not found")?;

    Ok(cont)
}

pub async fn start_container(channel: &mut ClientChannel, name: &str) -> Result<()> {
    let id = get_container_by_name(channel, name).await?.id;
    let _r = tonic::Request::new(cm_rpc::StartContainerRequest { id });
    let _r = channel.start(_r).await?;
    Ok(())
}

pub async fn stop_container(channel: &mut ClientChannel, name: &str, timeout: i64) -> Result<()> {
    let id = get_container_by_name(channel, name).await?.id;

    let stop_options = Some(cm_types::StopOptions {
        timeout,
        force: true,
        signal: String::from("SIGTERM"),
    });

    let _r = tonic::Request::new(cm_rpc::StopContainerRequest { id, stop_options });
    let _r = channel.stop(_r).await?;
    Ok(())
}

pub async fn remove_container(channel: &mut ClientChannel, name: &str, force: bool) -> Result<()> {
    let id = get_container_by_name(channel, name).await?.id;
    let _r = tonic::Request::new(cm_rpc::RemoveContainerRequest { id, force });
    let _r = channel.remove(_r).await?;
    Ok(())
}
