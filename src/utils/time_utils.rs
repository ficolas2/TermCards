use std::time::{SystemTime, UNIX_EPOCH};

pub fn now_s() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

pub fn format_until_duration(diff: i64) -> String {
    if diff <= 0 {
        return "now".to_string();
    }

    let minutes = ((diff as f64) / 60.0).round() as i64;
    let hours = ((diff as f64) / 3600.0).round() as i64;
    let days = ((diff as f64) / 86_400.0).round() as i64;

    if days > 0 {
        format!("in {} day{}", days, if days == 1 { "" } else { "s" })
    } else if hours > 0 {
        format!("in {} hour{}", hours, if hours == 1 { "" } else { "s" })
    } else if minutes > 0 {
        format!("in {} minute{}", minutes, if minutes == 1 { "" } else { "s" })
    } else {
        format!("in {} second{}", diff, if diff == 1 { "" } else { "s" })
    }
}

