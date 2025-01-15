use serde::de::{Error, Unexpected};
use serde::{Deserialize, Deserializer};
use serde_xml_rs::from_str;
use std::num::ParseIntError;
use std::path::PathBuf;
use std::str::from_utf8;
use std::str::FromStr;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use walkdir::WalkDir;

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
pub struct Resources {
    #[serde(rename = "Item")]
    pub items: Vec<Resource>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub enum Operations {
    Read,
    Write,
    ReadWrite,
    Execute,
    None,
}

#[derive(Debug, Deserialize, PartialEq)]
pub enum ResourceType {
    String,
    Integer,
    Float,
    Boolean,
    Opaque,
    Time,
    ObjectLink,
    UnsignedInteger,
    Corelink,
    Other,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Resource {
    #[serde(rename = "ID")]
    pub id: u16, // Number of resources is 'unbound' in XSD
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Operations", deserialize_with = "deserialize_operations")]
    pub operations: Operations,

    #[serde(
        rename = "MultipleInstances",
        deserialize_with = "deserialize_multiple_instances"
    )]
    pub has_multiple_instances: bool,
    #[serde(rename = "Mandatory", deserialize_with = "deserialize_mandatory")]
    pub is_mandatory: bool,
    #[serde(rename = "Type", deserialize_with = "deserialize_resource_type")]
    pub resource_type: ResourceType,
}

impl Resource {
    pub fn new(
        id: u16,
        name: String,
        operations: Operations,
        has_multiple_instances: bool,
        is_mandatory: bool,
        resource_type: ResourceType,
    ) -> Self {
        Self {
            id,
            name,
            operations,
            has_multiple_instances,
            is_mandatory,
            resource_type,
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
    #[serde(rename = "Resources", deserialize_with = "unwrap_resources_list")]
    pub resources: Vec<Resource>,
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

fn unwrap_resources_list<'de, D>(deserializer: D) -> Result<Vec<Resource>, D::Error>
where
    D: Deserializer<'de>,
{
    /// Represents <Resources>...</Resources>
    #[derive(Deserialize)]
    struct Resources {
        // default allows empty list
        #[serde(default, rename = "Item")]
        item: Vec<Resource>,
    }
    Ok(Resources::deserialize(deserializer)?.item)
}

fn deserialize_operations<'de, D>(deserializer: D) -> Result<Operations, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;

    match s.as_str() {
        "R" => Ok(Operations::Read),
        "W" => Ok(Operations::Write),
        "RW" => Ok(Operations::ReadWrite),
        "E" => Ok(Operations::Execute),
        _ => Ok(Operations::None),
    }
}

fn deserialize_resource_type<'de, D>(deserializer: D) -> Result<ResourceType, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;

    match s.as_str() {
        "String" => Ok(ResourceType::String),
        "Integer" => Ok(ResourceType::Integer),
        "Float" => Ok(ResourceType::Float),
        "Boolean" => Ok(ResourceType::Boolean),
        "Opaque" => Ok(ResourceType::Opaque),
        "Time" => Ok(ResourceType::Time),
        "Objlnk" => Ok(ResourceType::ObjectLink),
        "Unsigned Integer" => Ok(ResourceType::UnsignedInteger),
        "Corelnk" => Ok(ResourceType::Corelink),
        _ => Ok(ResourceType::Other),
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

async fn load(directories: &Vec<PathBuf>) -> anyhow::Result<Vec<Object>> {
    let mut objects = Vec::new();

    for directory in directories {
        for entry in WalkDir::new(directory) {
            let entry = entry?;
            if entry.file_type().is_file() {
                let f_name = entry.path().to_string_lossy();

                if f_name.ends_with(".xml") {
                    if let Ok(file) = File::open(entry.into_path()).await {
                        if let Ok(spec) = deserialize_spec_file(file).await {
                            for object in spec.objects {
                                objects.push(object);
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(objects)
}

pub struct Registry {
    directories: Vec<PathBuf>,
    pub objects: Vec<Object>,
}

impl Registry {
    pub async fn init(directories: Vec<PathBuf>) -> anyhow::Result<Registry> {
        let dir = directories.clone();
        let objects = load(&dir);
        let objects = objects.await?;
        let reg = Registry {
            directories,
            objects,
        };

        Ok(reg)
    }

    pub async fn reload(&mut self) -> anyhow::Result<()> {
        self.objects = load(&self.directories).await?;
        Ok(())
    }
}
