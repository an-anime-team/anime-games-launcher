pub fn pretty_bytes(bytes: u64) -> String {
    if bytes <= 1024 {
        format!("{bytes} B")
    } else if bytes <= 1024 * 1024 {
        format!("{:.2} KB", bytes as f64 / 1024.0)
    } else if bytes <= 1024 * 1024 * 1024 {
        format!("{:.2} MB", bytes as f64 / 1024.0 / 1024.0)
    } else if bytes <= 1024 * 1024 * 1024 * 1024 {
        format!("{:.2} GB", bytes as f64 / 1024.0 / 1024.0 / 1024.0)
    } else {
        format!("{bytes} B")
    }
}

pub fn pretty_seconds(mut seconds: u64) -> String {
    let hours = seconds / 3600;

    seconds -= hours * 3600;

    let hours = if hours < 10 {
        format!("0{hours}")
    } else {
        hours.to_string()
    };

    let minutes = seconds / 60;

    seconds -= minutes * 60;

    let minutes = if minutes < 10 {
        format!("0{minutes}")
    } else {
        minutes.to_string()
    };

    let seconds = if seconds < 10 {
        format!("0{seconds}")
    } else {
        seconds.to_string()
    };

    format!("{hours}:{minutes}:{seconds}")
}
