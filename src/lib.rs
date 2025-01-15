mod deserialize;
mod spec_files;

use deserialize::deserialize_mandatory;
use deserialize::deserialize_multiple_instances;
use deserialize::deserialize_operations;
use deserialize::deserialize_resource_type;
use deserialize::deserialize_unwrap_resources_list;
use deserialize::deserialize_version;
use serde::Deserialize;
use std::num::ParseIntError;
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version {
    major: u16,
    minor: u16,
}

impl Version {
    pub fn new(major: u16, minor: u16) -> Self {
        Self { major, minor }
    }

    fn parse_digit(num: Option<&str>) -> Result<u16, ParseVersionError> {
        if let Some(num) = num {
            let num: u16 = num.parse()?;
            Ok(num)
        } else {
            Err(ParseVersionError::new("NO_VALUE"))
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseVersionError {
    msg: String,
}

impl ParseVersionError {
    fn new(s: &str) -> Self {
        Self {
            msg: format!("Could not parse string: {}", s),
        }
    }
}

impl From<ParseIntError> for ParseVersionError {
    fn from(int_error: ParseIntError) -> Self {
        ParseVersionError::new(int_error.to_string().as_str())
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
            0 | 3.. => Err(Self::Err::new(s)),
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
    #[serde(
        rename = "Resources",
        deserialize_with = "deserialize_unwrap_resources_list"
    )]
    pub resources: Vec<Resource>,
}
#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct LwM2MSpec {
    #[serde(rename = "Object")]
    pub objects: Vec<Object>,
}

pub struct Registry {
    directories: Vec<PathBuf>,
    pub objects: Vec<Object>,
}

impl Registry {
    pub async fn init(directories: Vec<PathBuf>) -> anyhow::Result<Registry> {
        let dir = directories.clone();
        let objects = spec_files::load(&dir);
        let objects = objects.await?;
        let reg = Registry {
            directories,
            objects,
        };

        Ok(reg)
    }

    pub async fn reload(&mut self) -> anyhow::Result<()> {
        self.objects = spec_files::load(&self.directories).await?;
        Ok(())
    }

    pub fn has_object_id(&self, object_id: u16, version: Version) -> bool {
        self.objects
            .iter()
            .any(|o| o.object_id == object_id && o.object_version == version)
    }

    pub fn get_object_name(&self, object_id: u16, version: Version) -> Option<String> {
        let obj = self
            .objects
            .iter()
            .find(|o| o.object_id == object_id && o.object_version == version);
        if let Some(obj) = obj {
            return Some(obj.name.clone());
        }
        None
    }

    pub fn get_object_id_by_name_newest(&self, name: &str) -> Option<(u16, Version)> {
        let mut objs = self
            .objects
            .iter()
            .filter(|o| o.name == name)
            .collect::<Vec<&Object>>();
        objs.sort_by_key(|o| &o.object_version);
        if !objs.is_empty() {
            if let Some(obj) = objs.pop() {
                return Some((obj.object_id, obj.object_version));
            }
        }
        None
    }

    pub fn get_object_urn(&self, object_id: u16, version: Version) -> Option<String> {
        let obj = self
            .objects
            .iter()
            .find(|o| o.object_id == object_id && o.object_version == version);
        if let Some(obj) = obj {
            return Some(obj.object_urn.clone());
        }
        None
    }
}
