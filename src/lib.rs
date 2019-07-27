//! # pit
//!
//! pit is account management tool.
//!
//! ## Usage
//!
//! ```toml
//! [dependencies]
//! pit = "*"
//! ```
//!
//! ```sh
//! $ mkdir -p ~/.pit
//! $ echo "profile: default" > ~/.pit/pit.yaml
//! $ echo "twitter.com:" >> ~/.pit/default.yaml
//! $ echo "    username: foo" >> ~/.pit/default.yaml
//! $ echo "    password: bar" >> ~/.pit/default.yaml
//! ```
//!
//! ```rust
//! extern crate pit;
//!
//! use pit::Pit;
//!
//! fn main() {
//!     let p = Pit::new();
//!     let config = p.get("twitter.com");
//!     match config {
//!         None => {
//!             println!("not provide config value");
//!             return;
//!         },
//!         Some(_) => {},
//!     }
//!
//!     let config = config.unwrap();
//!     let username = config.get("username").unwrap();
//!     let password = config.get("password").unwrap();
//!     println!("username={}, password={}", username, password);
//! }
//! ```
//!

use std::fs::{File, Permissions, create_dir, set_permissions};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::io::prelude::*;
use std::io::{BufReader, BufWriter};
use std::collections::BTreeMap;
use yaml_rust::yaml::Yaml;
use yaml_rust::YamlLoader;

use dirs::home_dir;

#[derive(Clone, Debug, Default)]
pub struct Pit {
    directory: PathBuf,
    config_path: PathBuf,
    profile_path: PathBuf,
}

impl Pit {
    pub fn new() -> Pit {
        let homedir = home_dir().expect("fail home_dir");
        let pit_rootdir = homedir.join(Path::new(".pit"));
        let config_path = pit_rootdir.join(Path::new("pit.yaml"));
        let profile_path = pit_rootdir.join(Path::new("default.yaml"));
        Self {
            directory: pit_rootdir,
            config_path: config_path,
            profile_path: profile_path,
        }
    }

    pub fn set(self, _name: &str) {
        unimplemented!();
    }

    pub fn switch(&mut self, name: &str) -> Result<String, String> {
        let filename = format!("{}.yaml", name);
        self.profile_path = self.directory.join(Path::new(filename.as_str()));
        if !self.profile_path.exists() {
            return Err("profile not exists".to_string());
        }

        let config_name = self.get_config_name();
        let mut f = BufWriter::new(File::create(&self.config_path).expect("file create() error"));
        let c = format!("profile: {}", name);
        let _ = f.write_all(c.as_bytes());
        Ok(config_name)
    }

    pub fn get(mut self, name: &str) -> Option<BTreeMap<String, String>> {
        let config = match self.load() {
            Ok(v) => Some(v),
            Err(_) => {
                None
            },
        };
        if config.is_none() {
            return None;
        }
        let n = name.to_string();
        let config = config.unwrap();
        let profile = config.as_hash().expect("fail to top level object")
                           .get(&Yaml::String(n));
        let mut ret: BTreeMap<String, String> = BTreeMap::new();
        if profile.is_none() {
            return None;
        };
        if profile.unwrap().as_hash().is_none() {
            return None;
        }
        let ks = profile.unwrap().as_hash().unwrap();
        for (k, v) in ks.iter() {
            let ret_k = k.as_str().unwrap().to_string();
            let ret_v = v.as_str().unwrap().to_string();
            ret.insert(ret_k, ret_v);
        }
        Some(ret)
    }

    fn load(&mut self) -> Result<Yaml, String> {
        if !self.directory.exists() {
            match create_dir(self.directory.as_path()) {
                Ok(()) => {},
                Err(_) => {
                    return Err("invalid pit config".to_string());
                }
            }
            let _ = set_permissions(&self.directory, Permissions::from_mode(0o700));
        }

        if !self.config_path.exists() {
            let mut f = BufWriter::new(File::create(&self.config_path).expect("file create() error"));
            let default_profile = "profile: default";
            let _ = f.write_all(default_profile.as_bytes());
            let _ = set_permissions(&self.config_path, Permissions::from_mode(0o600));
        }

        let config_name = self.get_config_name();
        if config_name.is_empty() {
            return Err("invalid pit config".to_string());
        }
        let _ = self.switch(config_name.as_str());

        if !self.profile_path.exists() {
            let mut f = BufWriter::new(File::create(&self.profile_path).expect("file create() error"));
            let c = format!("--- {{}}");
            let _ = f.write_all(c.as_bytes());
            let _ = set_permissions(&self.profile_path, Permissions::from_mode(0o600));
        }

        let mut f = BufReader::new(File::open(&self.profile_path).expect("fail open file"));
        let mut b = String::new();
        let _ = f.read_to_string(&mut b);
        let mut docs = YamlLoader::load_from_str(&b).expect("yaml load fail");
        let doc = docs.pop();

        if doc.is_none() {
            return Err("invalid pit profile data".to_string());
        }
        Ok(doc.unwrap())
    }

    fn get_config_name(&self) -> String {
        let mut f = BufReader::new(File::open(&self.config_path).expect("fail open file"));
        let mut b = String::new();
        let _ = f.read_to_string(&mut b);
        let mut docs = YamlLoader::load_from_str(&b).expect("yaml load fail");
        docs.pop().expect("fail get config value")
            .as_hash().unwrap()
            .get(&Yaml::String("profile".to_string())).unwrap()
            .as_str().unwrap()
            .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::Pit;

    #[test]
    fn simple() {
        let mut p = Pit::new();
        let _ = p.switch("pit-rs-test");
        let config = p.clone().get("not exist key");
        assert_eq!(config, None);

        let config = p.get("test_key");
        assert!(config.is_some());
        assert_eq!(config.unwrap()["username"], "taro")
    }

    #[test]
    fn not_exists_key() {
        let mut p = Pit::new();
        let _ = p.switch("default");
        let config = p.get("not exist key");
        assert_eq!(config, None);
    }
}
