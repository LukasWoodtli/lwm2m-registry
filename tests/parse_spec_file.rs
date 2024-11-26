use lwm2m_registry::{deserialize_spec_file, Object};
use lwm2m_registry::{LwM2MSpec, Version};
use std::path::PathBuf;
use tokio::fs::File;

#[tokio::test]
async fn test_parse_spec_file() -> Result<(), Box<dyn std::error::Error>> {
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("tests/spec_files");
    d.push("0.xml");
    let file = File::open(d).await?;

    let expected = LwM2MSpec {
        objects: vec![Object {
            name: "LWM2M Security".to_string(),
            object_id: 0,
            object_urn: "urn:oma:lwm2m:oma:0:1.2".to_string(),
            object_version: Version::new(1, 2),
            lwm2m_version: Version::new(1, 1),
            has_multiple_instances: true,
            is_mandatory: true,
        }],
    };

    let actual = deserialize_spec_file(file).await?;
    assert_eq!(actual, expected);
    Ok(())
}
