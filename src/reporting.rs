use std::io::{stdout, Write};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};
use colored::Colorize;
use sysinfo::System;
use crate::sensors;

pub const CARRIAGE_RETURN: char = '\r';


pub fn watch_in_background(
    stop_temperature: Option<i64>,
    duration: Option<Duration>,
    system: &mut System,
    start_time: Instant,
    running: Arc<AtomicBool>,
) -> (Option<f32>, String) {
    let mut stop_reason = String::new();
    let one_second = Duration::from_secs(1);
    let mut iterations = 0;
    let mut average_cpu_temp = 0f32;

    while stop_reason.is_empty() {
        let temp = sensors::cpu_temp(system, true);
        if let Some(temp) = temp {
           average_cpu_temp += temp;
            if let Some(stop_temp) = stop_temperature {
                if temp > stop_temp as f32 {
                    stop_reason.push_str("Temperature exceeded");
                }
            }
        }


        if let Some(duration) = duration {
            if start_time.elapsed() > duration {
                stop_reason.push_str("Time up");
            }
        }

        print!("{}", prettify_output(duration, start_time, temp));
        stdout().flush().unwrap_or(());
        iterations += 1;
        thread::sleep(one_second);
    }
    running.store(false, Ordering::SeqCst);
    (Some(average_cpu_temp / (iterations as f32)), stop_reason)
}

fn prettify_output(
    duration: Option<Duration>,
    start_time: Instant,
    stop_temperature: Option<f32>
) -> String {
    let mut display_string = String::new();
    display_string.push(CARRIAGE_RETURN);

    if let Some(duration) = duration {
        let time_left_second = duration.as_secs() - start_time.elapsed().as_secs();
        let time_left = if time_left_second > 60 {
            format!(
                "🕛: {}m {}s",
                time_left_second / 60,
                time_left_second % 60
            )
        } else {
            format!("🕛: {}s", time_left_second)
        };
        display_string.push_str(time_left.as_str());
    }

    display_string.push_str(" 🌡️: ");

    if let Some(temp) = stop_temperature {
        let temp_text = if temp > 80.0 {
            format!("{}°C", temp).red().to_string()
        } else if temp > 60.0 {
            format!("{}°C", temp).yellow().to_string()
        } else {
            format!("{}°C", temp).green().to_string()
        };
        display_string.push_str(temp_text.as_str());
    }

    display_string

}