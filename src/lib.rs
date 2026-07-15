#![deny(missing_docs)]
//! This crate provides functionality to read LwM2M object specification files and use the contained
//! information in an application.
//! It also provides some functions to query the loaded specifications.
//! ```
//! # use std::path::PathBuf;
//! # use lwm2m_registry::Version;
//! # use crate::lwm2m_registry::Registry;
//! # let rt = tokio::runtime::Builder::new_current_thread().build().unwrap();
//! # rt.block_on(async {
//! let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
//! d.push("tests/spec_files");
//! let registry = Registry::init(vec![d]).await.unwrap();
//! assert!(registry.has_object_id(3, Version::new(1, 1)));
//! # })
//! ```
//!
//! # Logging
//!
//! This crate logs through the [`log`](https://docs.rs/log) facade; install a logger
//! implementation (e.g. `env_logger`) in your application to see the output.
//!
//! - `warn`: problems relevant for troubleshooting, such as spec files that are
//!   skipped because they can't be read or parsed, or directories that yield no
//!   spec files at all.
//! - `info`: a summary of how many objects were loaded.
//! - `debug`: details for developers, such as each scanned directory and each
//!   loaded object.
//! - `trace`: every spec file as it is read.
mod deserialize;
mod spec_files;

use deserialize::deserialize_mandatory;
use deserialize::deserialize_multiple_instances;
use deserialize::deserialize_operations;
use deserialize::deserialize_resource_type;
use deserialize::deserialize_unwrap_resources_list;
use deserialize::deserialize_version;
use serde::Deserialize;
use std::fmt;
use std::num::ParseIntError;
use std::path::PathBuf;
use std::str::FromStr;

/// This can represent a LwM2M version or an object version.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Version {
    major: u16,
    minor: u16,
}

impl Version {
    /// Create a new Version with a minor and mayor version number.
    pub fn new(major: u16, minor: u16) -> Self {
        Self { major, minor }
    }
}

/// Version assumed by the LwM2M specification when a spec file omits the
/// version elements.
impl Default for Version {
    fn default() -> Self {
        Version::new(1, 0)
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}

/// Error indicating that the version in a spec file could not be parsed.
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

impl fmt::Display for ParseVersionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.msg)
    }
}

impl std::error::Error for ParseVersionError {}

impl From<ParseIntError> for ParseVersionError {
    fn from(int_error: ParseIntError) -> Self {
        ParseVersionError::new(int_error.to_string().as_str())
    }
}

impl FromStr for Version {
    type Err = ParseVersionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let numbers = s.trim().split('.').collect::<Vec<_>>();
        match numbers.as_slice() {
            [major] => Ok(Version {
                major: major.parse()?,
                minor: 0,
            }),
            [major, minor] => Ok(Version {
                major: major.parse()?,
                minor: minor.parse()?,
            }),
            _ => Err(Self::Err::new(s)),
        }
    }
}

/// Operations that are allowed on a resource.
#[derive(Debug, Deserialize, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Operations {
    /// Resource can be only read.
    Read,
    /// Resource can be only written.
    Write,
    /// Resource can be read and written.
    ReadWrite,
    /// Resource can be only executed.
    Execute,
    /// No operations allowed on resource.
    None,
}

/// Indicates the type of resource.
#[derive(Debug, Deserialize, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ResourceType {
    /// The resource is a string (utf-8).
    String,
    /// The resource is a signed integer number.
    Integer,
    /// The resource is a floating point number.
    Float,
    /// The resource is a boolean value.
    Boolean,
    /// The resource is of opaque type.
    Opaque,
    /// The resource represents a time.
    Time,
    /// The resource links to an object.
    ObjectLink,
    /// The resource is an unsigned integer number.
    UnsignedInteger,
    /// The resource is of CoreLink format.
    Corelink,
    /// Unspecified resource type.
    Other,
}

/// A resource within an LwM2M object.
#[derive(Debug, Deserialize, PartialEq)]
pub struct Resource {
    /// The resource ID.
    #[serde(rename = "@ID")]
    pub id: u16, // Number of resources is 'unbound' in XSD
    /// The name of the resource.
    #[serde(rename = "Name")]
    pub name: String,
    /// The allowed operations for the resource.
    #[serde(rename = "Operations", deserialize_with = "deserialize_operations")]
    pub operations: Operations,
    /// Indicates if the resource has multiple instances.
    #[serde(
        rename = "MultipleInstances",
        deserialize_with = "deserialize_multiple_instances"
    )]
    pub has_multiple_instances: bool,
    #[serde(rename = "Mandatory", deserialize_with = "deserialize_mandatory")]
    /// Indicates if the resource is mandatory.
    pub is_mandatory: bool,
    /// The type of the resource.
    #[serde(rename = "Type", deserialize_with = "deserialize_resource_type")]
    pub resource_type: ResourceType,
}

impl Resource {
    /// Initialize a new resource.
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

/// Represents a LwM2M object as defined in a specification file
#[derive(Debug, Deserialize, PartialEq)]
pub struct Object {
    /// The name of the object.
    #[serde(rename = "Name")]
    pub name: String,
    /// The object ID.
    #[serde(rename = "ObjectID")]
    pub object_id: u16,
    /// The URN of the object.
    #[serde(rename = "ObjectURN")]
    pub object_urn: String,
    /// The object version.
    /// Defaults to 1.0 when the element is absent, as defined by the LwM2M specification.
    #[serde(
        rename = "ObjectVersion",
        deserialize_with = "deserialize_version",
        default
    )]
    pub object_version: Version,
    /// The LwM2M version where the object was introduced.
    /// Defaults to 1.0 when the element is absent, as defined by the LwM2M specification.
    #[serde(
        rename = "LWM2MVersion",
        deserialize_with = "deserialize_version",
        default
    )]
    pub lwm2m_version: Version,
    /// Indicates it the object can have multiple instances.
    #[serde(
        rename = "MultipleInstances",
        deserialize_with = "deserialize_multiple_instances"
    )]
    pub has_multiple_instances: bool,
    /// Indicates if the object is mandatory.
    #[serde(rename = "Mandatory", deserialize_with = "deserialize_mandatory")]
    pub is_mandatory: bool,
    /// The list of resources of the object.
    #[serde(
        rename = "Resources",
        deserialize_with = "deserialize_unwrap_resources_list"
    )]
    pub resources: Vec<Resource>,
}

/// Represents a LwM2M object specification file.
#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct LwM2MSpec {
    /** List of all specified objects in the file
        Note: Usually a spec file contains only one object definition.
    */
    #[serde(rename = "Object")]
    pub objects: Vec<Object>,
}

/** The registry reads spec files from a list of given directories and parses all found specification
    files. The retrieved data can then be queried for various information.
*/
pub struct Registry {
    directories: Vec<PathBuf>,
    /// All the objects that were retrieved from the specification files.
    pub objects: Vec<Object>,
}

impl Registry {
    /** Initialize a registry with a number of given directories.
        The directories are then walked and all XML files that are found are loaded and parsed.
    */
    pub async fn init(directories: Vec<PathBuf>) -> anyhow::Result<Registry> {
        let objects = spec_files::load(&directories).await?;
        Ok(Registry {
            directories,
            objects,
        })
    }

    /// Discard all the current objects and reload all files to populate the list of objects again.
    pub async fn reload(&mut self) -> anyhow::Result<()> {
        log::info!("Reloading registry");
        self.objects = spec_files::load(&self.directories).await?;
        Ok(())
    }

    /// Check if a given object ID with version exists.
    pub fn has_object_id(&self, object_id: u16, version: Version) -> bool {
        self.objects
            .iter()
            .any(|o| o.object_id == object_id && o.object_version == version)
    }

    /// Get the object name for a given object ID
    pub fn get_object_name(&self, object_id: u16, version: Version) -> Option<String> {
        self.get_object_by_id(object_id, version)
            .map(|o| o.name.clone())
    }

    /// Get the object for a given object ID with version.
    pub fn get_object_by_id(&self, object_id: u16, version: Version) -> Option<&Object> {
        self.objects
            .iter()
            .find(|o| o.object_id == object_id && o.object_version == version)
    }

    /// Get a resource by ID for a given object ID with version.
    pub fn get_resource_by_id(
        &self,
        object_id: u16,
        version: Version,
        resource_id: u16,
    ) -> Option<&Resource> {
        self.get_object_by_id(object_id, version)?
            .resources
            .iter()
            .find(|r| r.id == resource_id)
    }

    /** Get an object ID and it's version for a given name. Returns the object with the highest version. */
    pub fn get_object_id_by_name_newest(&self, name: &str) -> Option<(u16, Version)> {
        self.objects
            .iter()
            .filter(|o| o.name == name)
            .max_by_key(|o| o.object_version)
            .map(|o| (o.object_id, o.object_version))
    }

    /// Get the object URN for a given object ID with version
    pub fn get_object_urn(&self, object_id: u16, version: Version) -> Option<String> {
        self.get_object_by_id(object_id, version)
            .map(|o| o.object_urn.clone())
    }

    /// Get a resource name by ID for a given object ID with version.
    pub fn get_resource_name(
        &self,
        object_id: u16,
        version: Version,
        resource_id: u16,
    ) -> Option<String> {
        self.get_resource_by_id(object_id, version, resource_id)
            .map(|r| r.name.clone())
    }

    /// Get a resource ID by name for a given object ID with version.
    pub fn get_resource_id_by_name(
        &self,
        object_id: u16,
        version: Version,
        resource_name: &str,
    ) -> Option<u16> {
        self.get_object_by_id(object_id, version)?
            .resources
            .iter()
            .find(|r| r.name == resource_name)
            .map(|r| r.id)
    }

    /// Get a resources type by resource ID for a given object ID with version.
    pub fn get_resource_type(
        &self,
        object_id: u16,
        version: Version,
        resource_id: u16,
    ) -> Option<ResourceType> {
        self.get_resource_by_id(object_id, version, resource_id)
            .map(|r| r.resource_type)
    }

    /** Check if a resource can have multiple instances.
       It is retrieved by object ID with version and resource ID.
    */
    pub fn is_resource_multi_instance(
        &self,
        object_id: u16,
        version: Version,
        resource_id: u16,
    ) -> Option<bool> {
        self.get_resource_by_id(object_id, version, resource_id)
            .map(|r| r.has_multiple_instances)
    }

    /// Get all object ID's with their versions.
    pub fn get_object_ids(&self) -> Vec<(u16, Version)> {
        self.objects
            .iter()
            .map(|o| (o.object_id, o.object_version))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_from_str_major_only() {
        assert_eq!(Version::from_str("2"), Ok(Version::new(2, 0)));
    }

    #[test]
    fn test_version_from_str_major_minor() {
        assert_eq!(Version::from_str("1.2"), Ok(Version::new(1, 2)));
    }

    #[test]
    fn test_version_from_str_trims_whitespace() {
        assert_eq!(Version::from_str(" 1.1 "), Ok(Version::new(1, 1)));
    }

    #[test]
    fn test_version_from_str_too_many_parts() {
        assert_eq!(
            Version::from_str("1.2.3"),
            Err(ParseVersionError::new("1.2.3"))
        );
    }

    #[test]
    fn test_version_from_str_not_a_number() {
        assert!(Version::from_str("a.b").is_err());
        assert!(Version::from_str("").is_err());
        assert!(Version::from_str("1.x").is_err());
    }

    #[test]
    fn test_version_ordering() {
        assert!(Version::new(1, 2) > Version::new(1, 1));
        assert!(Version::new(2, 0) > Version::new(1, 9));
        assert_eq!(Version::new(1, 0), Version::new(1, 0));
    }

    #[test]
    fn test_version_display() {
        assert_eq!(Version::new(1, 2).to_string(), "1.2");
    }

    #[test]
    fn test_parse_version_error_display() {
        let error = ParseVersionError::new("1.2.3");
        assert_eq!(error.to_string(), "Could not parse string: 1.2.3");
    }

    #[test]
    fn test_parse_version_error_from_parse_int_error() {
        let int_error = "x".parse::<u16>().unwrap_err();
        let error = ParseVersionError::from(int_error);
        assert!(error.msg.contains("invalid digit"));
    }

    #[test]
    fn test_debug_formatting() {
        assert_eq!(
            format!("{:?}", Version::new(1, 0)),
            "Version { major: 1, minor: 0 }"
        );
        assert_eq!(format!("{:?}", Operations::ReadWrite), "ReadWrite");
        assert_eq!(format!("{:?}", ResourceType::Time), "Time");
    }
}
