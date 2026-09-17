use dioxus::prelude::*;
use crate::constants::TIMER_ACCENT;
//use dioxus::logger::tracing;

const TOTAL_TICKS: u64 = 12;

pub(crate) fn format_elapsed(total_seconds: u64) -> String {
    let h = total_seconds / 3600;
    let m = (total_seconds % 3600) / 60;
    let s = total_seconds % 60;
    if h > 0 {
        format!("{h:02}:{m:02}:{s:02}")
    } else {
        format!("{m:02}:{s:02}")
    }
}

#[component]
pub fn ProgressRing(elapsed_seconds: ReadSignal<u64>) -> Element {
    let elapsed_seconds = elapsed_seconds();

    let box_radius = 100f64;
    let radius = 0.92 * box_radius;
    let circumference = 2.0 * std::f64::consts::PI * radius;
    let laps_completed = (elapsed_seconds / crate::clock_window::LAP_SECONDS).min(TOTAL_TICKS);
    let progress_fraction = (elapsed_seconds % crate::clock_window::LAP_SECONDS) as f64 / crate::clock_window::LAP_SECONDS as f64;
    //if progress_fraction == 0.0 && elapsed_seconds != 0 {progress_fraction = 1.0};
    let dash_offset = circumference * (1.0 - progress_fraction);
    let track_color = "var(--color-border)";
    //let animates = elapsed_seconds % LAP_SECONDS != 0;

    let tick_positions: Vec<(f64, f64, bool)> = (0..TOTAL_TICKS)
        .map(|i| {
            let angle_deg = i as f64 * (360.0 / TOTAL_TICKS as f64);
            let angle_rad = (angle_deg - 90.0).to_radians();
            let tick_radius = 0.83 * radius;
            let x = box_radius + tick_radius * angle_rad.cos();
            let y = box_radius + tick_radius * angle_rad.sin();
            (x, y, i < laps_completed)
        })
        .collect();

    rsx! {
        svg {
            width: "{2.*box_radius}",
            height: "{2.*box_radius}",
            view_box: "0 0 {2.*box_radius} {2.*box_radius}",

            circle {
                cx: "{box_radius}", cy: "{box_radius}", r: "{radius}",
                fill: "none",
                stroke: "{track_color}",
                stroke_width: "6",
            }
            circle {
                cx: "{box_radius}", cy: "{box_radius}", r: "{radius}",
                fill: "none",
                stroke: "{TIMER_ACCENT}",
                stroke_width: "6",
                stroke_linecap: "round",
                stroke_dasharray: "{circumference}",
                stroke_dashoffset: "{dash_offset}",
                transform: "rotate(-90 {box_radius} {box_radius})",
                style: "" //"transition: stroke-dashoffset 1s linear;"
            }
            for (x, y, lit) in tick_positions.iter() {
                circle {
                    cx: "{x}", cy: "{y}", r: "3.5",
                    fill: if *lit { TIMER_ACCENT } else { track_color },
                }
            }
        }
    }
}