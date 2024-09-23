pub fn pretty_bytes(bytes: u64) -> String {
    if bytes <= 1024 {
        format!("{bytes} B")
    } else if bytes <= 1024 * 1024 {
        format!("{:.2} KiB", bytes as f64 / 1024.0)
    } else if bytes <= 1024 * 1024 * 1024 {
        format!("{:.2} MiB", bytes as f64 / 1024.0 / 1024.0)
    } else if bytes <= 1024 * 1024 * 1024 * 1024 {
        format!("{:.2} GiB", bytes as f64 / 1024.0 / 1024.0 / 1024.0)
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

pub fn pretty_frequency(hz: u64, is_ram: bool) -> String {
    let (value, unit) = if hz < 1000 {
        (hz as f64, "Hz")
    } else if hz < 1_000_000 {
        (hz as f64 / 1000.0, "kHz")
    } else if hz < 1_000_000_000 || is_ram {
        (hz as f64 / 1_000_000.0, "MHz")
    } else {
        (hz as f64 / 1_000_000_000.0, "GHz")
    };
    format!("{} {}", value, unit)
}
