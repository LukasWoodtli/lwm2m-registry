use lwm2m_registry::LwM2MSpec;
use lwm2m_registry::{deserialize_spec_file, Object};
use std::path::PathBuf;
use tokio::fs::File;

#[tokio::test]
async fn test_parse_spec_file() -> Result<(), Box<dyn std::error::Error>> {
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("tests/spec_files");
    d.push("0.xml");
    let file = File::open(d).await?;

    let expected = LwM2MSpec {
        object: Object {
            name: "LWM2M Security".to_string(),
        },
    };

    let actual = deserialize_spec_file(file).await?;
    assert_eq!(actual.object, expected.object);
    Ok(())
}
