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

#[cfg(test)]
mod tests {
    use super::*;
    use serde::de::value::{Error as ValueError, StrDeserializer, U32Deserializer};
    use serde::de::IntoDeserializer;

    fn str_deserializer(s: &str) -> StrDeserializer<'_, ValueError> {
        s.into_deserializer()
    }

    fn non_string_deserializer() -> U32Deserializer<ValueError> {
        42u32.into_deserializer()
    }

    #[test]
    fn test_deserialize_non_string_input() {
        assert!(deserialize_version(non_string_deserializer()).is_err());
        assert!(deserialize_multiple_instances(non_string_deserializer()).is_err());
        assert!(deserialize_mandatory(non_string_deserializer()).is_err());
        assert!(deserialize_operations(non_string_deserializer()).is_err());
        assert!(deserialize_resource_type(non_string_deserializer()).is_err());
        assert!(deserialize_unwrap_resources_list(non_string_deserializer()).is_err());
    }

    #[test]
    fn test_deserialize_version() {
        assert_eq!(
            deserialize_version(str_deserializer("1.2")),
            Ok(Version::new(1, 2))
        );
    }

    #[test]
    fn test_deserialize_version_invalid() {
        assert!(deserialize_version(str_deserializer("not a version")).is_err());
        assert!(deserialize_version(str_deserializer("1.2.3")).is_err());
    }

    #[test]
    fn test_deserialize_multiple_instances() {
        assert_eq!(
            deserialize_multiple_instances(str_deserializer("Multiple")),
            Ok(true)
        );
        assert_eq!(
            deserialize_multiple_instances(str_deserializer("Single")),
            Ok(false)
        );
    }

    #[test]
    fn test_deserialize_multiple_instances_invalid() {
        assert!(deserialize_multiple_instances(str_deserializer("Both")).is_err());
    }

    #[test]
    fn test_deserialize_mandatory() {
        assert_eq!(
            deserialize_mandatory(str_deserializer("Mandatory")),
            Ok(true)
        );
        assert_eq!(
            deserialize_mandatory(str_deserializer("Optional")),
            Ok(false)
        );
    }

    #[test]
    fn test_deserialize_mandatory_invalid() {
        assert!(deserialize_mandatory(str_deserializer("Maybe")).is_err());
    }

    #[test]
    fn test_deserialize_operations() {
        let cases = [
            ("R", Operations::Read),
            ("W", Operations::Write),
            ("RW", Operations::ReadWrite),
            ("E", Operations::Execute),
            ("", Operations::None),
            ("X", Operations::None),
        ];
        for (input, expected) in cases {
            assert_eq!(
                deserialize_operations(str_deserializer(input)),
                Ok(expected)
            );
        }
    }

    #[test]
    fn test_deserialize_resource_type() {
        let cases = [
            ("String", ResourceType::String),
            ("Integer", ResourceType::Integer),
            ("Float", ResourceType::Float),
            ("Boolean", ResourceType::Boolean),
            ("Opaque", ResourceType::Opaque),
            ("Time", ResourceType::Time),
            ("Objlnk", ResourceType::ObjectLink),
            ("Unsigned Integer", ResourceType::UnsignedInteger),
            ("Corelnk", ResourceType::Corelink),
            ("Something Else", ResourceType::Other),
        ];
        for (input, expected) in cases {
            assert_eq!(
                deserialize_resource_type(str_deserializer(input)),
                Ok(expected)
            );
        }
    }
}
