fn main() -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
        std::env::set_var(
            "PROTOC",
            protoc_bin_vendored::protoc_bin_path().expect("protoc binary not found"),
        );
    }
    tonic_build::configure()
        .compile_protos(&["proto/analytics.proto", "proto/ml.proto"], &["proto"])?;
    Ok(())
}
