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
            if let Ok(offset_result) = std::panic::catch_unwind(|| {
                js_eval("new Date().getTimezoneOffset()")
            }) {
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
    format!("{} {}", formatted_time, tz_abbr)
}

/// Format time range with timezone indicator in 12-hour format  
pub fn format_time_range_with_timezone(start_time: &str, end_time: Option<&str>, timezone_signal: ReadSignal<String>) -> String {
    let tz_abbr = timezone_signal.get();
    match end_time {
        Some(end) => {
            let start_12 = convert_to_12_hour_format(start_time);
            let end_12 = convert_to_12_hour_format(end);
            format!("{} - {} {}", start_12, end_12, tz_abbr)
        },
        None => {
            let start_12 = convert_to_12_hour_format(start_time);
            format!("{} {}", start_12, tz_abbr)
        }
    }
}