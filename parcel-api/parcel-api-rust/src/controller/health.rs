use axum::{http::header, response::IntoResponse};

const BANNER: &str = "parcel-api";

pub async fn status() -> impl IntoResponse {
    "👋 parcel-api is on air"
}

pub async fn health() -> impl IntoResponse {
    let body = format!(
        "{BANNER}\n\nMemory:\n{}\n\nVersion:\ndev\n",
        memory_summary()
    );
    (
        [
            (header::CONTENT_TYPE, "text/plain; charset=utf-8"),
            (header::CACHE_CONTROL, "no-cache"),
        ],
        body,
    )
}

fn memory_summary() -> String {
    #[cfg(target_os = "linux")]
    {
        if let Ok(contents) = std::fs::read_to_string("/proc/self/statm") {
            let page_size = 4096u64;
            let parts: Vec<&str> = contents.split_whitespace().collect();
            if parts.len() >= 2 {
                let total_mb = parts[0].parse::<u64>().unwrap_or(0) * page_size / 1_048_576;
                let rss_mb = parts[1].parse::<u64>().unwrap_or(0) * page_size / 1_048_576;
                return format!("total: {total_mb}mb, resident: {rss_mb}mb");
            }
        }
    }
    "memory stats unavailable on this platform".to_string()
}
