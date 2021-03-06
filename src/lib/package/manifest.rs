//! Package manifest files.

use self::version::Constraint;
use super::{
    resolution::{DirectRes, IndexRes, Resolution},
    *,
};
use failure::{Error, ResultExt};
use indexmap::IndexMap;
use semver::Version;
use std::{path::PathBuf, str::FromStr};
use toml;
use url::Url;
use url_serde;
use util::errors::*;

// TODO: Package aliasing. Have dummy alias files in the root target folder.
//
// e.g. to alias `me/lightyear` with default root module `Me.Lightyear` as the module
// `Yeet.Lightyeet`, in the target folder, we make the following file in the proper directory
// (directory won't matter for Blodwen/Idris 2):
//
// ```idris
// module Yeet.Lightyeet
//
// import public Me.Lightyear
// ```
//
// Behind the scenes, we build this as its own package with the package it's aliasing as
// its only dependency, throw it in the global cache, and add this to the import dir of the root
// package instead of the original.
//
// I guess this also means that each package should declare their (root) module(s), so that we
// can identify conflicts ahead of time without having to guess that it's always gonna be Group.Name
//
// With this in place, we can safely avoid module namespace conflicts.

#[derive(Deserialize, Debug)]
pub struct Manifest {
    package: PackageInfo,
    #[serde(default = "IndexMap::new")]
    pub dependencies: IndexMap<Name, DepReq>,
    #[serde(default = "IndexMap::new")]
    pub dev_dependencies: IndexMap<Name, DepReq>,
    targets: Targets,
    #[serde(default)]
    workspace: IndexMap<Name, String>,
}

impl Manifest {
    pub fn summary(&self) -> Summary {
        let pid = PackageId::new(self.package.name.clone(), Resolution::Root);
        Summary::new(pid, self.package.version.clone())
    }

    pub fn version(&self) -> &Version {
        &self.package.version
    }
}

impl FromStr for Manifest {
    type Err = Error;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        toml::from_str(raw)
            .context(ErrorKind::InvalidManifestFile)
            .map_err(Error::from)
    }
}

#[derive(Deserialize, Debug)]
struct PackageInfo {
    name: Name,
    version: Version,
    authors: Vec<String>,
    license: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum DepReq {
    Registry(Constraint),
    RegLong {
        con: Constraint,
        registry: IndexRes,
    },
    Local {
        path: PathBuf,
    },
    Git {
        #[serde(with = "url_serde")]
        git: Url,
        #[serde(default)]
        #[serde(flatten)]
        spec: PkgGitSpecifier,
    },
}

impl DepReq {
    pub fn into_dep(self, def_index: IndexRes, n: Name) -> (PackageId, Constraint) {
        match self {
            DepReq::Registry(c) => {
                let pi = PackageId::new(n, def_index.into());
                (pi, c)
            }
            DepReq::RegLong { con, registry } => {
                let pi = PackageId::new(n, registry.into());
                (pi, con)
            }
            DepReq::Local { path } => {
                let res = DirectRes::Dir { url: path };
                let pi = PackageId::new(n, res.into());
                (pi, Constraint::any())
            }
            DepReq::Git { git, spec } => {
                let res = DirectRes::Git {
                    repo: git,
                    tag: spec,
                };
                let pi = PackageId::new(n, res.into());
                (pi, Constraint::any())
            }
        }
    }
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum PkgGitSpecifier {
    Branch(String),
    Commit(String),
    Tag(String),
}

impl FromStr for PkgGitSpecifier {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut s = s.splitn(2, '=');
        let fmt = s.next().unwrap();
        let spec = s
            .next()
            .ok_or_else(|| ErrorKind::InvalidSourceUrl)?
            .to_string();

        match fmt {
            "branch" => Ok(PkgGitSpecifier::Branch(spec)),
            "commit" => Ok(PkgGitSpecifier::Commit(spec)),
            "tag" => Ok(PkgGitSpecifier::Tag(spec)),
            _ => Err(ErrorKind::InvalidSourceUrl)?,
        }
    }
}

impl fmt::Display for PkgGitSpecifier {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PkgGitSpecifier::Branch(a) => write!(f, "branch={}", a),
            PkgGitSpecifier::Commit(a) => write!(f, "branch={}", a),
            PkgGitSpecifier::Tag(a) => write!(f, "branch={}", a),
        }
    }
}

impl Default for PkgGitSpecifier {
    fn default() -> Self {
        PkgGitSpecifier::Branch("master".to_string())
    }
}

#[derive(Deserialize, Debug)]
struct Targets {
    lib: Option<Target>,
    #[serde(default = "Vec::new")]
    bin: Vec<BinTarget>,
    #[serde(default = "Vec::new")]
    test: Vec<Target>,
    #[serde(default = "Vec::new")]
    bench: Vec<Target>,
}

#[derive(Deserialize, Debug)]
struct Target {
    path: PathBuf,
}

#[derive(Deserialize, Debug)]
struct BinTarget {
    name: String,
    // For binaries, benches, and tests, this should point to a file with a Main module.
    main: PathBuf,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_manifest() {
        let manifest = r#"
[package]
name = 'ring_ding/test'
version = '1.0.0'
authors = ['me']
license = 'MIT'

[dependencies]
'awesome/a' = '>= 1.0.0 < 2.0.0'
'cool/b' = { git = 'https://github.com/super/cool', tag = "v1.0.0" }
'great/c' = { path = 'file://here/right/now' }

[dev_dependencies]
'ayy/x' = '2.0'

[[targets.bin]]
name = 'bin1'
main = 'src/bin/Here.idr'

[targets.lib]
path = "src/lib/"
"#;

        assert!(Manifest::from_str(manifest).is_ok());
    }
}
