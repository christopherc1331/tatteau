/// Timezone utility for formatting times with proper timezone detection and 12-hour format
use leptos::prelude::*;

/// Get timezone abbreviation - returns a signal that updates when client-side hydration completes
pub fn get_timezone_abbreviation() -> ReadSignal<String> {
    // Create a signal that starts with a default and updates client-side
    let (timezone, set_timezone) = signal("UTC".to_string());

    // Effect that only runs on the client side
    Effect::new(move |_| {
        #[cfg(feature = "hydrate")]
        {
            use wasm_bindgen::prelude::*;

            #[wasm_bindgen]
            extern "C" {
                #[wasm_bindgen(js_name = eval)]
                fn js_eval(code: &str) -> JsValue;
            }

            // Try to get the actual timezone from Intl API
            if let Ok(timezone_result) = std::panic::catch_unwind(|| {
                js_eval("Intl.DateTimeFormat().resolvedOptions().timeZone")
            }) {
                if let Some(timezone_name) = timezone_result.as_string() {
                    set_timezone.set(timezone_name_to_abbreviation(&timezone_name));
                    return;
                }
            }

            // Fallback: get timezone offset and convert to abbreviation
            if let Ok(offset_result) =
                std::panic::catch_unwind(|| js_eval("new Date().getTimezoneOffset()"))
            {
                if let Some(offset_minutes) = offset_result.as_f64() {
                    let offset_hours = (-offset_minutes / 60.0) as i32;
                    set_timezone.set(offset_to_timezone_abbr(offset_hours));
                }
            }
        }
    });

    timezone
}

/// Convert timezone name to common abbreviation
#[cfg(feature = "hydrate")]
fn timezone_name_to_abbreviation(tz_name: &str) -> String {
    match tz_name {
        "America/New_York" => "EST".to_string(),
        "America/Chicago" => "CST".to_string(),
        "America/Denver" => "MST".to_string(),
        "America/Los_Angeles" => "PST".to_string(),
        "America/Phoenix" => "MST".to_string(),
        "Europe/London" => "GMT".to_string(),
        "Europe/Paris" | "Europe/Berlin" | "Europe/Rome" => "CET".to_string(),
        "Asia/Tokyo" => "JST".to_string(),
        "Asia/Shanghai" => "CST".to_string(),
        "Australia/Sydney" => "AEDT".to_string(),
        _ => {
            // Extract common abbreviation patterns
            if let Some(last_part) = tz_name.split('/').last() {
                last_part.to_uppercase()
            } else {
                "Local".to_string()
            }
        }
    }
}

/// Convert UTC offset to timezone abbreviation
#[cfg(feature = "hydrate")]
fn offset_to_timezone_abbr(offset_hours: i32) -> String {
    match offset_hours {
        -12 => "BIT".to_string(),
        -11 => "SST".to_string(),
        -10 => "HST".to_string(),
        -9 => "AKST".to_string(),
        -8 => "PST".to_string(),
        -7 => "MST".to_string(),
        -6 => "CST".to_string(),
        -5 => "EST".to_string(),
        -4 => "AST".to_string(),
        0 => "UTC".to_string(),
        1 => "CET".to_string(),
        2 => "EET".to_string(),
        9 => "JST".to_string(),
        _ => format!("UTC{:+}", offset_hours),
    }
}

/// Convert 24-hour time to 12-hour format
pub fn convert_to_12_hour_format(time_24: &str) -> String {
    if let Some((hour_str, minute_str)) = time_24.split_once(':') {
        if let Ok(hour) = hour_str.parse::<u32>() {
            let (hour_12, period) = if hour == 0 {
                (12, "AM")
            } else if hour < 12 {
                (hour, "AM")
            } else if hour == 12 {
                (12, "PM")
            } else {
                (hour - 12, "PM")
            };

            // Only show minutes if they're not 00
            if minute_str == "00" {
                return format!("{} {}", hour_12, period);
            } else {
                return format!("{}:{} {}", hour_12, minute_str, period);
            }
        }
    }
    time_24.to_string() // Fallback to original if parsing fails
}

/// Format time with timezone indicator in 12-hour format
pub fn format_time_with_timezone(time: &str, timezone_signal: ReadSignal<String>) -> String {
    let tz_abbr = timezone_signal.get();
    let formatted_time = if time == "All Day" {
        "All Day".to_string()
    } else {
        convert_to_12_hour_format(time)
    };

    // Only append timezone if it's not UTC (since we're showing local time)
    if tz_abbr != "UTC" {
        format!("{} {}", formatted_time, tz_abbr)
    } else {
        formatted_time
    }
}

/// Format time range with timezone indicator in 12-hour format  
pub fn format_time_range_with_timezone(
    start_time: &str,
    end_time: Option<&str>,
    timezone_signal: ReadSignal<String>,
) -> String {
    let tz_abbr = timezone_signal.get();
    let formatted_range = match end_time {
        Some(end) => {
            let start_12 = convert_to_12_hour_format(start_time);
            let end_12 = convert_to_12_hour_format(end);
            format!("{} - {}", start_12, end_12)
        }
        None => convert_to_12_hour_format(start_time),
    };

    // Only append timezone if it's not UTC (since we're showing local time)
    if tz_abbr != "UTC" {
        format!("{} {}", formatted_range, tz_abbr)
    } else {
        formatted_range
    }
}

/// Convert UTC datetime string to local timezone and format for display
#[cfg(feature = "hydrate")]
pub fn convert_utc_datetime_to_local(utc_datetime: &str) -> String {
    use wasm_bindgen::prelude::*;

    // Use JavaScript to convert UTC to local time
    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_name = eval)]
        fn js_eval(code: &str) -> JsValue;
    }

    // Create JavaScript code to parse and format the datetime
    let js_code = format!(
        r#"
        (function() {{
            try {{
                let date = new Date('{}');
                if (isNaN(date.getTime())) {{
                    return '{}';
                }}
                return date.toLocaleString();
            }} catch(e) {{
                return '{}';
            }}
        }})()
        "#,
        utc_datetime, utc_datetime, utc_datetime
    );

    if let Ok(result) = std::panic::catch_unwind(|| js_eval(&js_code)) {
        if let Some(formatted) = result.as_string() {
            return formatted;
        }
    }

    // Fallback to original string if parsing fails
    utc_datetime.to_string()
}

#[cfg(not(feature = "hydrate"))]
pub fn convert_utc_datetime_to_local(utc_datetime: &str) -> String {
    // Server-side fallback - just return the original string
    utc_datetime.to_string()
}

/// Format datetime for booking display with proper timezone conversion
pub fn format_datetime_for_booking(datetime: &str, timezone_signal: ReadSignal<String>) -> String {
    let tz_abbr = timezone_signal.get();

    // Clean up the datetime string - remove any existing timezone indicators
    let clean_datetime = datetime
        .replace(" UTC", "")
        .replace("UTC", "")
        .trim()
        .to_string();

    // Convert UTC datetime to local time
    #[cfg(feature = "hydrate")]
    {
        use wasm_bindgen::prelude::*;

        #[wasm_bindgen]
        extern "C" {
            #[wasm_bindgen(js_name = eval)]
            fn js_eval(code: &str) -> JsValue;
        }

        // Ensure we have a proper UTC timestamp format
        let utc_datetime = if clean_datetime.ends_with('Z') {
            clean_datetime.clone()
        } else if clean_datetime.len() >= 19
            && clean_datetime.contains('-')
            && clean_datetime.contains(':')
        {
            // Standard timestamp format, add Z for UTC
            format!("{}Z", clean_datetime)
        } else {
            // Not a full timestamp, return as-is with timezone
            return format!(
                "{} {}",
                clean_datetime,
                if tz_abbr != "UTC" {
                    tz_abbr
                } else {
                    "".to_string()
                }
            )
            .trim()
            .to_string();
        };

        // JavaScript code to convert UTC to local time with proper formatting
        let js_code = format!(
            r#"
            (function() {{
                try {{
                    let date = new Date('{}');
                    if (isNaN(date.getTime())) {{
                        return null;
                    }}
                    
                    // Format as: "Month DD, YYYY at H:MM AM/PM"
                    const options = {{
                        month: 'long',
                        day: 'numeric', 
                        year: 'numeric',
                        hour: 'numeric',
                        minute: '2-digit',
                        hour12: true
                    }};
                    
                    return date.toLocaleString('en-US', options);
                }} catch(e) {{
                    return null;
                }}
            }})()
            "#,
            utc_datetime
        );

        if let Ok(result) = std::panic::catch_unwind(|| js_eval(&js_code)) {
            if let Some(formatted) = result.as_string() {
                // Add local timezone abbreviation (not UTC)
                if tz_abbr != "UTC" {
                    return format!("{} {}", formatted, tz_abbr);
                } else {
                    return formatted;
                }
            }
        }
    }

    // Server-side or fallback formatting
    #[cfg(not(feature = "hydrate"))]
    {
        // Simple fallback formatting for server-side
        if clean_datetime.len() >= 19 {
            // Try to parse and format nicely
            if let Some(date_part) = clean_datetime.split(' ').next() {
                if let Some(time_part) = clean_datetime.split(' ').nth(1) {
                    let formatted_date = format_date_for_booking(date_part);
                    let formatted_time = convert_to_12_hour_format(&time_part[..5]);
                    return format!(
                        "{} at {} {}",
                        formatted_date,
                        formatted_time,
                        if tz_abbr != "UTC" {
                            tz_abbr
                        } else {
                            "".to_string()
                        }
                    )
                    .trim()
                    .to_string();
                }
            }
        }
    }

    // Final fallback
    format!(
        "{} {}",
        clean_datetime,
        if tz_abbr != "UTC" {
            tz_abbr
        } else {
            "".to_string()
        }
    )
    .trim()
    .to_string()
}

/// Format date for booking display (date only, no time)
pub fn format_date_for_booking(date: &str) -> String {
    // Try to parse and format the date nicely
    if date.len() >= 10 && date.contains('-') {
        // Assume YYYY-MM-DD format
        let parts: Vec<&str> = date.split('-').collect();
        if parts.len() == 3 {
            if let (Ok(year), Ok(month), Ok(day)) = (
                parts[0].parse::<u32>(),
                parts[1].parse::<u32>(),
                parts[2].parse::<u32>(),
            ) {
                let month_name = match month {
                    1 => "January",
                    2 => "February",
                    3 => "March",
                    4 => "April",
                    5 => "May",
                    6 => "June",
                    7 => "July",
                    8 => "August",
                    9 => "September",
                    10 => "October",
                    11 => "November",
                    12 => "December",
                    _ => return date.to_string(),
                };
                return format!("{} {}, {}", month_name, day, year);
            }
        }
    }

    // Fallback to original if parsing fails
    date.to_string()
}
