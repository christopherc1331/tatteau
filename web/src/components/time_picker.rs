use leptos::prelude::*;
use wasm_bindgen::JsCast;

#[component]
pub fn TimePicker(
    value: RwSignal<String>,
    #[prop(optional)] placeholder: Option<&'static str>,
    #[prop(optional)] label: Option<&'static str>,
) -> impl IntoView {
    let hours = RwSignal::new(9);
    let minutes = RwSignal::new(0);
    let am_pm = RwSignal::new("AM".to_string());

    // Parse initial value
    Effect::new(move |_| {
        let current_value = value.get();
        if !current_value.is_empty() && current_value.contains(':') {
            let parts: Vec<&str> = current_value.split(':').collect();
            if parts.len() == 2 {
                if let Ok(hour) = parts[0].parse::<i32>() {
                    if let Ok(minute) = parts[1].parse::<i32>() {
                        if hour == 0 {
                            hours.set(12);
                            am_pm.set("AM".to_string());
                        } else if hour < 12 {
                            hours.set(hour);
                            am_pm.set("AM".to_string());
                        } else if hour == 12 {
                            hours.set(12);
                            am_pm.set("PM".to_string());
                        } else {
                            hours.set(hour - 12);
                            am_pm.set("PM".to_string());
                        }
                        minutes.set(minute);
                    }
                }
            }
        }
    });

    // Update value when any part changes
    Effect::new(move |_| {
        let h = hours.get();
        let m = minutes.get();
        let period = am_pm.get();

        let hour_24 = if period == "AM".to_string() {
            if h == 12 {
                0
            } else {
                h
            }
        } else {
            if h == 12 {
                12
            } else {
                h + 12
            }
        };

        value.set(format!("{:02}:{:02}", hour_24, m));
    });

    let hour_options = (1..=12)
        .map(|h| {
            view! {
                <option value={h.to_string()} selected=move || hours.get() == h>
                    {h.to_string()}
                </option>
            }
        })
        .collect_view();

    let minute_options = (0..60)
        .step_by(15)
        .map(|m| {
            view! {
                <option value={m.to_string()} selected=move || minutes.get() == m>
                    {format!("{:02}", m)}
                </option>
            }
        })
        .collect_view();

    view! {
        <div class="time-picker">
            {label.map(|l| view! {
                <label class="time-picker-label">{l}</label>
            })}
            <div class="time-picker-controls">
                <select class="time-picker-hours"
                        on:change=move |e| {
                            let target = e.target().unwrap();
                            let select = target.dyn_into::<web_sys::HtmlSelectElement>().unwrap();
                            if let Ok(h) = select.value().parse::<i32>() {
                                hours.set(h);
                            }
                        }>
                    {hour_options}
                </select>
                <span class="time-picker-separator">":"</span>
                <select class="time-picker-minutes"
                        on:change=move |e| {
                            let target = e.target().unwrap();
                            let select = target.dyn_into::<web_sys::HtmlSelectElement>().unwrap();
                            if let Ok(m) = select.value().parse::<i32>() {
                                minutes.set(m);
                            }
                        }>
                    {minute_options}
                </select>
                <select class="time-picker-ampm"
                        on:change=move |e| {
                            let target = e.target().unwrap();
                            let select = target.dyn_into::<web_sys::HtmlSelectElement>().unwrap();
                            am_pm.set(select.value());
                        }>
                    <option value="AM" selected=move || am_pm.get() == "AM".to_string()>"AM"</option>
                    <option value="PM" selected=move || am_pm.get() == "PM".to_string()>"PM"</option>
                </select>
            </div>
        </div>
    }
}
