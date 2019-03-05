use wearte_config::{read_config_file, Config};

use std::fs;
use wearte_config::config_file_path;

// TODO: document
pub fn when_changed() {
    println!(
        "cargo:rerun-if-changed={}",
        config_file_path().to_str().unwrap()
    );

    let file = read_config_file();
    let config = Config::new(&file);

    // rerun when dir change for add files
    println!(
        "cargo:rerun-if-changed={}",
        config.get_dir().to_str().unwrap()
    );
    let mut stack = vec![config.get_dir().clone()];
    loop {
        if let Some(dir) = stack.pop() {
            for entry in fs::read_dir(dir).expect("valid directory") {
                let path = entry.expect("valid directory entry").path();
                if path.is_dir() {
                    stack.push(path);
                } else {
                    // rerun when file change
                    println!("cargo:rerun-if-changed={}", path.to_str().unwrap());
                }
            }
        } else {
            break;
        }
    }
}
