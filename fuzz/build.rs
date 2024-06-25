use std::io::Result;

fn main() -> Result<()> {
    let proto_base_path = std::path::PathBuf::from("proto");

    let protos = &[
        proto_base_path.join("sysvars.proto"),
        proto_base_path.join("invoke.proto"),
    ];

    protos
        .iter()
        .for_each(|proto| println!("cargo:rerun-if-changed={}", proto.display()));

    prost_build::compile_protos(protos, &[proto_base_path])?;

    Ok(())
}
