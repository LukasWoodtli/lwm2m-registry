use lwm2m_registry::{Registry, Version};
use std::path::PathBuf;

async fn load_test_registry() -> Result<Registry, Box<dyn std::error::Error>> {
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("tests/spec_files");

    Ok(Registry::init(vec![d]).await?)
}

#[tokio::test]
async fn test_load_registry() -> Result<(), Box<dyn std::error::Error>> {
    let registry = load_test_registry().await?;

    assert_eq!(6, registry.objects.len());
    Ok(())
}

#[tokio::test]
async fn test_has_object_id() -> Result<(), Box<dyn std::error::Error>> {
    let registry = load_test_registry().await?;
    assert!(registry.has_object_id(1, Version::new(1, 1)));
    assert!(!registry.has_object_id(1, Version::new(2, 1)));
    Ok(())
}
