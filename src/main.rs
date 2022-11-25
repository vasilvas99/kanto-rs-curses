#[cfg(unix)]
#[tokio::main]
async fn main() -> kantocurses::kanto_api::Result<()>{
    let mut c = kantocurses::kanto_api::get_connection("/run/container-management/container-management.sock").await?;
    let r = kantocurses::kanto_api::list_containers(&mut c).await?;
    println!("{:#?}", r);
    Ok(())
}