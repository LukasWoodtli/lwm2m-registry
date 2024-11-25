use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize};
use serde_xml_rs::from_str;
use std::str::from_utf8;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Object {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "ObjectID")]
    pub object_id: u16,
    #[serde(rename = "ObjectURN")]
    pub object_urn: String,
    #[serde(rename = "LWM2MVersion")]
    pub lwm2m_version: String,
    #[serde(
        rename = "MultipleInstances",
        deserialize_with = "deserialize_multiple_instances"
    )]
    pub has_multiple_instances: bool,
    #[serde(rename = "Mandatory", deserialize_with = "deserialize_mandatory")]
    pub is_mandatory: bool,
}
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct LwM2MSpec {
    #[serde(rename = "Object")]
    pub objects: Vec<Object>,
}

fn deserialize_multiple_instances<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;

    match s.as_str() {
        "Multiple" => Ok(true),
        "Single" => Ok(false),
        _ => Err(Error::unknown_variant(&s, &["Multiple", "Single"])),
    }
}

fn deserialize_mandatory<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;

    match s.as_str() {
        "Mandatory" => Ok(true),
        "Optional" => Ok(false),
        _ => Err(Error::unknown_variant(&s, &["Mandatory", "Optional"])),
    }
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
