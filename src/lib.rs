use serde::{Deserialize, Serialize};
use serde_xml_rs::from_str;
use std::str::from_utf8;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Object {
    pub name: String,
}
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct LwM2MSpec {
    pub object: Object,
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
