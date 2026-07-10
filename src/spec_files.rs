use crate::{LwM2MSpec, Object};
use log::{debug, info, trace, warn};
use serde_xml_rs::from_str;
use std::path::PathBuf;
use std::str::from_utf8;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use walkdir::WalkDir;

pub async fn load(directories: &[PathBuf]) -> anyhow::Result<Vec<Object>> {
    let mut objects = Vec::new();

    for directory in directories {
        debug!("Scanning directory {} for spec files", directory.display());
        let mut files_loaded = 0;
        for entry in WalkDir::new(directory) {
            let entry = entry?;
            let is_xml_file = entry.file_type().is_file()
                && entry
                    .path()
                    .extension()
                    .is_some_and(|e| e.eq_ignore_ascii_case("xml"));

            if is_xml_file {
                let path = entry.into_path();
                trace!("Reading spec file {}", path.display());
                match File::open(&path).await {
                    Ok(file) => match deserialize_spec_file(file).await {
                        Ok(spec) => {
                            for object in &spec.objects {
                                debug!(
                                    "Loaded object {} '{}' version {} from {}",
                                    object.object_id,
                                    object.name,
                                    object.object_version,
                                    path.display()
                                );
                            }
                            files_loaded += 1;
                            objects.extend(spec.objects);
                        }
                        Err(e) => {
                            warn!("Ignoring unparsable spec file {}: {}", path.display(), e)
                        }
                    },
                    Err(e) => warn!("Ignoring unreadable spec file {}: {}", path.display(), e),
                }
            }
        }
        if files_loaded == 0 {
            warn!(
                "No spec files loaded from directory {}",
                directory.display()
            );
        }
    }
    info!(
        "Loaded {} objects from {} directories",
        objects.len(),
        directories.len()
    );
    Ok(objects)
}

pub async fn deserialize_spec_file(
    mut file: File,
) -> Result<LwM2MSpec, Box<dyn std::error::Error>> {
    let mut contents = vec![];
    file.read_to_end(&mut contents).await?;

    let str = from_utf8(contents.as_slice())?;

    let item: LwM2MSpec = from_str(str)?;

    Ok(item)
}

#[cfg(test)]
mod tests {

    use crate::spec_files::deserialize_spec_file;
    use crate::{Operations, Resource, ResourceType, Version};
    use std::path::PathBuf;
    use tokio::fs::File;

    #[tokio::test]
    async fn test_parse_spec_file() -> Result<(), Box<dyn std::error::Error>> {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("tests/spec_files/lwm2m_1_2");
        d.push("0-1_2.xml");
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
        assert_eq!(object.resources.len(), 31);
        let num_resources_to_check = expected_first_resources.len();
        assert_eq!(
            object.resources[0..num_resources_to_check],
            expected_first_resources
        );

        Ok(())
    }
}
