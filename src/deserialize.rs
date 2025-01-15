use crate::{Operations, Resource, ResourceType, Version};
use serde::de::{Error, Unexpected};
use serde::{Deserialize, Deserializer};
use std::str::FromStr;

pub(crate) fn deserialize_version<'de, D>(deserializer: D) -> Result<Version, D::Error>
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

pub(crate) fn deserialize_multiple_instances<'de, D>(deserializer: D) -> Result<bool, D::Error>
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

pub(crate) fn deserialize_mandatory<'de, D>(deserializer: D) -> Result<bool, D::Error>
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

pub(crate) fn deserialize_unwrap_resources_list<'de, D>(
    deserializer: D,
) -> Result<Vec<Resource>, D::Error>
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

pub(crate) fn deserialize_operations<'de, D>(deserializer: D) -> Result<Operations, D::Error>
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

pub(crate) fn deserialize_resource_type<'de, D>(deserializer: D) -> Result<ResourceType, D::Error>
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
