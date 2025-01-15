# A simple LwM2M Registry written in Rust

This crate provides functionality to read LwM2M object specification files and use the contained
information in an application.

## Example

```rust
use std::path::PathBuf;
use tokio_test;
use lwm2m_registry::Version;
use crate::lwm2m_registry::Registry;
tokio_test::block_on(async {
let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
d.push("tests/spec_files");
let registry = Registry::init(vec![d]).await.unwrap();
assert!(registry.has_object_id(3, Version::new(1, 1)));
})
```
## Links

Documentation: [docs.rs/lwm2m-registry](https://docs.rs/lwm2m-registry/latest/lwm2m_registry/)

Crate: [crates.io](https://crates.io/crates/lwm2m-registry)
