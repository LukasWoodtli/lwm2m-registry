use lwm2m_registry::Registry;
use std::path::PathBuf;

#[tokio::test]
async fn test_load_registry() -> Result<(), Box<dyn std::error::Error>> {
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("tests/spec_files");

    let registry = Registry::init(vec![d]).await?;

    assert_eq!(2, registry.objects.len());
    Ok(())
}
