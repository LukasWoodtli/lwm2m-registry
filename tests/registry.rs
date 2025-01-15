use lwm2m_registry::ResourceType::Opaque;
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

#[tokio::test]
async fn test_get_object_id_by_name_newest() -> Result<(), Box<dyn std::error::Error>> {
    let registry = load_test_registry().await?;
    let res = registry.get_object_id_by_name_newest("LwM2M Server");
    assert_eq!(res, Some((1, Version::new(1, 2))));
    Ok(())
}

#[tokio::test]
async fn test_get_object_id_by_name_newest_not_found() -> Result<(), Box<dyn std::error::Error>> {
    let registry = load_test_registry().await?;
    let res = registry.get_object_id_by_name_newest("Unknown Object");
    assert_eq!(res, None);
    Ok(())
}

#[tokio::test]
async fn test_get_object_urn() -> Result<(), Box<dyn std::error::Error>> {
    let registry = load_test_registry().await?;
    let res = registry.get_object_urn(3, Version::new(1, 1));
    assert_eq!(res, Some("urn:oma:lwm2m:oma:3:1.1".to_string()));
    Ok(())
}

#[tokio::test]
async fn test_get_object_urn_not_found() -> Result<(), Box<dyn std::error::Error>> {
    let registry = load_test_registry().await?;
    let res = registry.get_object_urn(99, Version::new(1, 1));
    assert_eq!(res, None);
    Ok(())
}

#[tokio::test]
async fn test_get_resource_name() -> Result<(), Box<dyn std::error::Error>> {
    let registry = load_test_registry().await?;
    let res = registry.get_resource_name(3, Version::new(1, 1), 0);
    assert_eq!(res, Some("Manufacturer".to_string()));
    Ok(())
}

#[tokio::test]
async fn test_get_resource_name_not_found() -> Result<(), Box<dyn std::error::Error>> {
    let registry = load_test_registry().await?;
    let res = registry.get_resource_name(3, Version::new(1, 1), 99);
    assert_eq!(res, None);
    Ok(())
}

#[tokio::test]
async fn test_get_resource_id_by_name() -> Result<(), Box<dyn std::error::Error>> {
    let registry = load_test_registry().await?;
    let res = registry.get_resource_id_by_name(3, Version::new(1, 2), "Factory Reset");
    assert_eq!(res, Some(5));
    Ok(())
}

#[tokio::test]
async fn test_get_resource_id_by_name_not_found() -> Result<(), Box<dyn std::error::Error>> {
    let registry = load_test_registry().await?;
    let res = registry.get_resource_id_by_name(3, Version::new(1, 2), "Unknown Resource");
    assert_eq!(res, None);
    Ok(())
}

#[tokio::test]
async fn test_get_resource_type() -> Result<(), Box<dyn std::error::Error>> {
    let registry = load_test_registry().await?;
    let res = registry.get_resource_type(0, Version::new(1, 1), 5);
    assert_eq!(res, Some(Opaque));
    Ok(())
}

#[tokio::test]
async fn test_get_resource_type_not_found() -> Result<(), Box<dyn std::error::Error>> {
    let registry = load_test_registry().await?;
    let res = registry.get_resource_type(0, Version::new(1, 1), 99);
    assert_eq!(res, None);
    Ok(())
}

#[tokio::test]
async fn test_is_resource_multi_instance_true() -> Result<(), Box<dyn std::error::Error>> {
    let registry = load_test_registry().await?;
    let res = registry.is_resource_multi_instance(3, Version::new(1, 1), 6);
    assert_eq!(res, Some(true));
    Ok(())
}

#[tokio::test]
async fn test_is_resource_multi_instance_false() -> Result<(), Box<dyn std::error::Error>> {
    let registry = load_test_registry().await?;
    let res = registry.is_resource_multi_instance(3, Version::new(1, 1), 0);
    assert_eq!(res, Some(false));
    Ok(())
}

#[tokio::test]
async fn test_is_resource_multi_instance_not_found() -> Result<(), Box<dyn std::error::Error>> {
    let registry = load_test_registry().await?;
    let res = registry.is_resource_multi_instance(0, Version::new(1, 1), 777);
    assert_eq!(res, None);
    Ok(())
}

#[tokio::test]
async fn test_get_object_ids() -> Result<(), Box<dyn std::error::Error>> {
    let registry = load_test_registry().await?;
    let mut res = registry.get_object_ids();
    assert_eq!(res.len(), 6);

    let mut expected = vec![
        (0, Version::new(1, 1)),
        (1, Version::new(1, 1)),
        (3, Version::new(1, 1)),
        (0, Version::new(1, 2)),
        (1, Version::new(1, 2)),
        (3, Version::new(1, 2)),
    ];

    res.sort();
    expected.sort();
    assert_eq!(res, expected);

    Ok(())
}
