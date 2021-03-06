//! Configuration for Indices.

use failure::{Error, ResultExt};
use package::resolution::IndexRes;
use std::str::FromStr;
use toml;
use util::errors::ErrorKind;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct IndexConfig {
    index: IndexConfInner,
}

impl FromStr for IndexConfig {
    type Err = Error;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        toml::from_str(raw)
            .context(ErrorKind::InvalidIndex)
            .map_err(Error::from)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct IndexConfInner {
    secure: bool,
    dependencies: Vec<IndexRes>,
}

impl Default for IndexConfInner {
    fn default() -> Self {
        IndexConfInner {
            secure: false,
            dependencies: vec![],
        }
    }
}
