use std::io::{stdout, Write};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};
use colored::Colorize;
use sysinfo::System;
use crate::sensors;

pub const CARRIAGE_RETURN: char = '\r';

pub struct BackgroundReport {
    pub average_cpu_temp: Option<f32>,
    pub min_cpu_temp: Option<f32>,
    pub max_cpu_temp: Option<f32>,
}

pub fn watch_in_background(
    stop_temperature: Option<i64>,
    duration: Option<Duration>,
    system: &mut System,
    start_time: Instant,
    running: Arc<AtomicUsize>,
) -> BackgroundReport {
    let mut iterations = 0;
    let mut average_cpu_temp = 0f32;
    let mut min_cpu_temp = 999.9f32;
    let mut max_cpu_temp = 0f32;


    while running.load(Ordering::Relaxed) == 0 {
        let temp = sensors::cpu_temp(system, true);
        if let Some(temp) = temp {

            if temp > max_cpu_temp {
                max_cpu_temp = temp;
            }

            if temp < min_cpu_temp {
                min_cpu_temp = temp;
            }

            average_cpu_temp += temp;


            if let Some(stop_temp) = stop_temperature {
                if temp > stop_temp as f32 {
                    running.store(2, Ordering::Relaxed)
                }
            }
        }


        if let Some(duration) = duration {
            if start_time.elapsed() > duration {
                running.store(1, Ordering::Relaxed)
            }
        }

        print!("{} ", prettify_output(duration, start_time, temp));
        stdout().flush().unwrap();
        iterations += 1;
    }

    BackgroundReport {
        average_cpu_temp: if average_cpu_temp == 0.0 { None } else { Some(average_cpu_temp / iterations as f32) },
        min_cpu_temp: if min_cpu_temp == 999.9 { None } else { Some(min_cpu_temp) },
        max_cpu_temp: if max_cpu_temp == 0.0  { None } else { Some(max_cpu_temp) },
    }
}

fn prettify_output(
    duration: Option<Duration>,
    start_time: Instant,
    stop_temperature: Option<f32>
) -> String {
    let mut display_string = String::new();
    display_string.push(CARRIAGE_RETURN);

    let time_left = match duration {
        Some(duration) => duration.as_secs() - start_time.elapsed().as_secs(),
        None => start_time.elapsed().as_secs(),
    };

    let time_string = if time_left > 60 {
        format!(
            "🕛: {}m {}s",
            time_left / 60,
            time_left % 60
        )
    } else {
        format!("🕛: {}s", time_left)
    };

    display_string.push_str(time_string.as_str());
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