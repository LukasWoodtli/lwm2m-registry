use serde::de::{Error, Unexpected};
use serde::{Deserialize, Deserializer};
use serde_xml_rs::from_str;
use std::num::ParseIntError;
use std::str::from_utf8;
use std::str::FromStr;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

#[derive(Debug, PartialEq)]
pub struct Version {
    major: u16,
    minor: u16,
}

impl Version {
    pub fn new(major: u16, minor: u16) -> Self {
        Self { major, minor }
    }

    fn parse_digit(num: Option<&str>) -> Result<u16, ParseVersionError> {
        let num = num.ok_or(ParseVersionError)?;
        let num: u16 = num.parse()?;
        Ok(num)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseVersionError;

impl From<ParseIntError> for ParseVersionError {
    fn from(_: ParseIntError) -> Self {
        ParseVersionError
    }
}

impl FromStr for Version {
    type Err = ParseVersionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut numbers = s.trim().split('.');
        let count = numbers.clone().count();
        match count {
            1 => Ok(Version {
                major: Self::parse_digit(numbers.next())?,
                minor: 0,
            }),
            2 => Ok(Version {
                major: Self::parse_digit(numbers.next())?,
                minor: Self::parse_digit(numbers.next())?,
            }),
            0 | 3.. => Ok(Version {
                major: 99,
                minor: 99,
            }),
        }
    }
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Object {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "ObjectID")]
    pub object_id: u16,
    #[serde(rename = "ObjectURN")]
    pub object_urn: String,
    #[serde(rename = "ObjectVersion", deserialize_with = "deserialize_version")]
    pub object_version: Version,
    #[serde(rename = "LWM2MVersion", deserialize_with = "deserialize_version")]
    pub lwm2m_version: Version,
    #[serde(
        rename = "MultipleInstances",
        deserialize_with = "deserialize_multiple_instances"
    )]
    pub has_multiple_instances: bool,
    #[serde(rename = "Mandatory", deserialize_with = "deserialize_mandatory")]
    pub is_mandatory: bool,
}
#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct LwM2MSpec {
    #[serde(rename = "Object")]
    pub objects: Vec<Object>,
}

fn deserialize_version<'de, D>(deserializer: D) -> Result<Version, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;

    let version = Version::from_str(&s);
    match version {
        Ok(v) => Ok(v),
        _ => Err(D::Error::invalid_value(
            Unexpected::Str(&s),
            &"a valid version string",
        )),
    }
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
