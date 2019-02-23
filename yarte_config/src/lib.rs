#[macro_use]
extern crate serde_derive;

use std::{
    env, fs,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub struct Config {
    pub dirs: Vec<PathBuf>,
}

impl Config {
    pub fn new(s: &str) -> Config {
        let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
        let default_dirs = vec![root.join("templates")];

        let raw: RawConfig =
            toml::from_str(&s).expect(&format!("invalid TOML in {}", CONFIG_FILE_NAME));

        let dirs = match raw.main {
            Some(Main { dir }) => dir.map_or(default_dirs, |v| {
                v.iter().map(|dir| root.join(dir)).collect()
            }),
            None => default_dirs,
        };

        Config { dirs }
    }

    pub fn find_template(&self, path: &str, start_at: Option<&Path>) -> PathBuf {
        if let Some(root) = start_at {
            let relative = root.with_file_name(path);
            if relative.exists() {
                return relative;
            }
        }

        for dir in &self.dirs {
            let rooted = dir.join(path);
            if rooted.exists() {
                return rooted;
            }
        }

        panic!(
            "template {:?} not found in directories {:?}",
            path, self.dirs
        )
    }
}

#[derive(Deserialize)]
struct RawConfig<'d> {
    #[serde(borrow)]
    main: Option<Main<'d>>,
}

#[derive(Deserialize)]
struct Main<'a> {
    #[serde(borrow)]
    dir: Option<Vec<&'a str>>,
}

pub fn read_config_file() -> String {
    let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let filename = root.join(CONFIG_FILE_NAME);
    if filename.exists() {
        fs::read_to_string(&filename)
            .expect(&format!("unable to read {}", filename.to_str().unwrap()))
    } else {
        "".to_string()
    }
}

static CONFIG_FILE_NAME: &str = "yarte.toml";

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::path::{Path, PathBuf};

    #[test]
    fn test_default_config() {
        let mut root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
        root.push("templates");
        let config = Config::new("");
        assert_eq!(config.dirs, vec![root]);
    }

    #[test]
    fn test_config_dirs() {
        let mut root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
        root.push("tpl");
        let config = Config::new("[main]\ndir = [\"tpl\"]");
        assert_eq!(config.dirs, vec![root]);
    }

    fn assert_eq_rooted(actual: &Path, expected: &str) {
        let mut root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
        root.push("templates");
        let mut inner = PathBuf::new();
        inner.push(expected);
        assert_eq!(actual.strip_prefix(root).unwrap(), inner);
    }

    #[test]
    fn find_absolute() {
        let config = Config::new("");
        let root = config.find_template("a.html", None);
        let path = config.find_template("sub/a.html", Some(&root));
        assert_eq_rooted(&path, "sub/a.html");
    }

    #[test]
    #[should_panic]
    fn find_relative_nonexistent() {
        let config = Config::new("");
        let root = config.find_template("a.html", None);
        config.find_template("b.html", Some(&root));
    }

    #[test]
    fn find_relative() {
        let config = Config::new("");
        let root = config.find_template("sub/a.html", None);
        let path = config.find_template("c.html", Some(&root));
        assert_eq_rooted(&path, "sub/c.html");
    }

    #[test]
    fn find_relative_sub() {
        let config = Config::new("");
        let root = config.find_template("sub/a.html", None);
        let path = config.find_template("sub1/d.html", Some(&root));
        assert_eq_rooted(&path, "sub/sub1/d.html");
    }
}
