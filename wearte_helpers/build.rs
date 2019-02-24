use version_check::is_nightly;

fn main() {
    if is_nightly().unwrap_or(false) {
        println!("cargo:rustc-cfg=wearte_nightly");
    }
}
