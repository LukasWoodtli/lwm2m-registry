use lwm2m_registry::Version;
use lwm2m_registry::{deserialize_spec_file, Operations, Resource, ResourceType};
use std::path::PathBuf;
use tokio::fs::File;

#[tokio::test]
async fn test_parse_spec_file() -> Result<(), Box<dyn std::error::Error>> {
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("tests/spec_files");
    d.push("0.xml");
    let file = File::open(d).await?;

    /* checking only the first few resources */
    let expected_first_resources = vec![
        Resource::new(
            0,
            "LWM2M  Server URI".to_string(),
            Operations::None,
            false,
            true,
            ResourceType::String,
        ),
        Resource::new(
            1,
            "Bootstrap-Server".to_string(),
            Operations::None,
            false,
            true,
            ResourceType::Boolean,
        ),
        Resource::new(
            2,
            "Security Mode".to_string(),
            Operations::None,
            false,
            true,
            ResourceType::Integer,
        ),
        Resource::new(
            3,
            "Public Key or Identity".to_string(),
            Operations::None,
            false,
            true,
            ResourceType::Opaque,
        ),
        Resource::new(
            4,
            "Server Public Key".to_string(),
            Operations::None,
            false,
            true,
            ResourceType::Opaque,
        ),
        Resource::new(
            5,
            "Secret Key".to_string(),
            Operations::None,
            false,
            true,
            ResourceType::Opaque,
        ),
    ];

    let actual = deserialize_spec_file(file).await?;
    assert_eq!(actual.objects.len(), 1);
    let object = actual.objects.first().unwrap();
    assert_eq!(object.name, "LWM2M Security".to_string());
    assert_eq!(object.object_id, 0);
    assert_eq!(object.object_urn, "urn:oma:lwm2m:oma:0:1.2".to_string());
    assert_eq!(object.object_version, Version::new(1, 2));
    assert_eq!(object.lwm2m_version, Version::new(1, 1));
    assert!(object.has_multiple_instances);
    assert!(object.is_mandatory);
    assert_eq!(object.resources.items.len(), 31);
    assert_eq!(object.resources.items.len(), 31);
    let num_resources_to_check = expected_first_resources.len();
    assert_eq!(
        object.resources.items[0..num_resources_to_check],
        expected_first_resources
    );

    Ok(())
}
