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

#[tokio::test]
async fn test_get_object_name() -> Result<(), Box<dyn std::error::Error>> {
    let registry = load_test_registry().await?;
    let name = registry.get_object_name(3, Version::new(1, 1));
    assert_eq!(name, Some("Device".to_string()));
    Ok(())
}

#[tokio::test]
async fn test_get_object_name_not_found() -> Result<(), Box<dyn std::error::Error>> {
    let registry = load_test_registry().await?;
    let name = registry.get_object_name(9, Version::new(2, 0));
    assert_eq!(name, None);
    Ok(())
}
