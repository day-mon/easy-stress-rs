use std::fmt::format;
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
) {
    let one_second = Duration::new(1, 0);
    loop {
        let temp = sensors::cpu_temp(system, true);
        if let Some(temp) = temp {
            if temp > stop_temperature.unwrap() as f32 {
                break;
            }
        }

        if let Some(duration) = duration {
            if start_time.elapsed() > duration {
                break;
            }
        }

        print!("{}", prettify_output(duration, start_time, temp));
        stdout().flush().unwrap();
        thread::sleep(one_second);
    }
    running.store(false, Ordering::SeqCst);
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
            format!(" 🕛: {}s", time_left_second)
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