//! Development & Localhost Environment Detection Service

/// Returns `true` if the application is running in local development mode or localhost.
///
/// Detection strategy:
/// 1. Compile-time check: `APP_ENV` or `ENV` in `.env` set to `"local"`, `"localhost"`, `"dev"`, `"development"`
/// 2. Browser runtime check: `window.location.hostname` matching `"localhost"`, `"127.0.0.1"`, or `*.local`
/// 3. Cargo debug build check: `cfg!(debug_assertions)`
pub fn is_dev_env() -> bool {
    // 1. Compile-time environment variable inspection from .env
    let app_env = option_env!("APP_ENV").or(option_env!("ENV")).unwrap_or("");
    if app_env == "local" || app_env == "localhost" || app_env == "dev" || app_env == "development"
    {
        return true;
    }

    // 2. Browser runtime hostname check
    #[cfg(target_arch = "wasm32")]
    if let Some(win) = web_sys::window() {
        if let Ok(hostname) = win.location().hostname() {
            if hostname == "localhost"
                || hostname == "127.0.0.1"
                || hostname == "0.0.0.0"
                || hostname.ends_with(".local")
            {
                return true;
            }
        }
    }

    // 3. Fallback to debug assertions (active during `trunk serve` / `cargo build`)
    cfg!(debug_assertions)
}
