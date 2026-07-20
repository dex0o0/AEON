use std::time::SystemTime;

fn main() {
    let now = SystemTime::now();
    let date = chrono::DateTime::<chrono::Utc>::from(now).format("");

    println!("cargo:rustc-env=BUILD_TIME={}", date);
}
