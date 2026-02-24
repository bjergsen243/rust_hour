fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_root = "../../proto";

    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile_protos(
            &[
                &format!("{proto_root}/auth.proto"),
                &format!("{proto_root}/monitor.proto"),
                &format!("{proto_root}/alert.proto"),
            ],
            &[proto_root],
        )?;

    Ok(())
}
