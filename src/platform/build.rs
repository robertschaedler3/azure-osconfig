const MODEL_VERSION: u32 = 15;

fn main() {
    if std::env::var("BUILD_NUMBER").is_ok() {
        return;
    }

    let build_date = std::env::var("BUILD_DATE")
        .unwrap_or_else(|_| chrono::Local::now().format("%Y%m%d").to_string());
    let revision = std::env::var("BUILD_REVISION").unwrap_or_else(|_| "dev".to_string());
    let build_number = format!("{}.{}", build_date, revision);

    println!("cargo:rustc-env=BUILD_NUMBER={}", build_number);

    // Azure OSConfig <model version>;<major>.<minor>.<patch>.<build>
    let version = env!("CARGO_PKG_VERSION");
    let client_name = format!(
        "Azure OSConfig {};{}.{}",
        MODEL_VERSION, version, build_number
    );

    println!("cargo:rustc-env=PLATFORM_CLIENT={}", client_name);
}
