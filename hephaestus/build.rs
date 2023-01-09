fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("proto/hephaestus.proto")?;
    tonic_build::compile_protos("proto/hermes.proto")?;
    Ok(())
}