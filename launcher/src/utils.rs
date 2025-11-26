/// Generate pretty time output from given seconds.
/// 
/// ```
/// assert_eq!(pretty_seconds(1),    "00:00:01");
/// assert_eq!(pretty_seconds(60),   "00:01:00");
/// assert_eq!(pretty_seconds(3600), "01:00:00");
/// ```
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

/// Generate pretty size output.
/// Second tuple value should be used for further
/// translation.
/// 
/// ```
/// let pretty = pretty_bytes(123456);
/// 
/// assert_eq!(pretty.0, 1234.56);
/// assert_eq!(pretty.1, "kb");
/// ```
pub fn pretty_bytes(bytes: u64) -> (f64, &'static str) {
    let units = [
        (1_000_000_000_000, "gb"),
        (1_000_000_000,     "gb"),
        (1_000_000,         "mb"),
        (1_000,             "kb")
    ];

    for (value, unit) in units {
        if bytes >= value {
            return (
                (bytes as f64) / (value as f64),
                unit
            );
        }
    }

    (bytes as f64, "b")
}

/// Generate pretty frequency output.
/// Second tuple value should be used for further
/// translation.
/// 
/// ```
/// let pretty = pretty_frequency(123456);
/// 
/// assert_eq!(pretty.0, 1234.56);
/// assert_eq!(pretty.1, "khz");
/// ```
pub fn pretty_frequency(hz: u64) -> (f64, &'static str) {
    let units = [
        (1_000_000_000, "ghz"),
        (1_000_000,     "mhz"),
        (1_000,         "khz")
    ];

    for (value, unit) in units {
        if hz >= value {
            return (
                (hz as f64) / (value as f64),
                unit
            );
        }
    }

    (hz as f64, "hz")
}
