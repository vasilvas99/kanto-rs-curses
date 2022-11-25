fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(false)
        .include_file("mod.rs")
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .compile(
            &["api/services/containers/containers.proto"],
            &["container-management/containerm/"],
        )?;
    Ok(())
}
