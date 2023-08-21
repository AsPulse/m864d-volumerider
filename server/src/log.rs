use chrono::Utc;

pub fn log_time() -> String {
    Utc::now().format("[%Y/%m/%d %H:%M:%S]").to_string()
}

pub fn log_time_role(role: &str) -> String {
    format!("{}<{}>", log_time(), role)
}