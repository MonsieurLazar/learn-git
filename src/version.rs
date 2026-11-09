pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn target() -> &'static str {
    if let Some(target) = option_env!("BUILD_TARGET")
        && !target.is_empty()
    {
        return target;
    }

    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("linux", "x86_64") => "x86_64-unknown-linux-gnu",
        ("linux", "aarch64") => "aarch64-unknown-linux-gnu",
        ("macos", "x86_64") => "x86_64-apple-darwin",
        ("macos", "aarch64") => "aarch64-apple-darwin",
        ("windows", "x86_64") => "x86_64-pc-windows-msvc",
        _ => "unknown",
    }
}
