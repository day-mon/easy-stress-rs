pub mod stressors;
pub mod sensors;
mod reporting;
pub mod work;

use std::fmt::{Error, format};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};
use requestty::{ListItem, Question};
use sysinfo::{CpuExt, DiskExt, System, SystemExt};
use crate::reporting::watch_in_background;

fn main() {
    let mut sys = System::new_all();

    // Display system information:s
    println!("Hello {:?}!", sys.host_name().unwrap());
    println!("OS: {:?} v{:?}", sys.long_os_version().unwrap(), sys.kernel_version().unwrap());
    println!("CPU: {:?} cores {:?} threads", sys.physical_core_count().unwrap(),  sys.cpus().len());
    println!("Memory: {:?} GBs", sys.total_memory() / 1024 / 1024 / 1024);

    const CPU_QUESTION_INDEXES: [i32; 3] = [0, 3, 4];

    loop {
        let questions = [
            Question::select("main_question")
                .message("What would you like to test?")
                .choices(["CPU", "GPU", "All (Separate)", "All (Together)"])
                .build(),
            Question::select("cpu_question")
                .message("How many CPU(s) would you like to use?")
                .choices((1..=sys.cpus().len()).map(|cpu| format!("{} CPU(s)", cpu)).collect::<Vec<String>>())
                .when(|ans: &requestty::Answers| {
                    let index_chosen = ans["main_question"].as_list_item().unwrap().index as i32;
                    CPU_QUESTION_INDEXES.contains(&index_chosen)
                })
                .build(),
            Question::multi_select("how_test")
                .message("How would you like the test to terminate? (Will terminate when any condition is met in this order)")
                .choices(get_termination_options(&mut sys))
                .validate(|ans, _ | {
                    if ans.iter().filter(|&&a| a).count() < 1 {
                        Err("You must choose at least one option.".into())
                    } else {
                        Ok(())
                    }
                })
                .build(),
            Question::int("duration")
                .message("How long would you like the test to be? (In Minutes)")
                .when(|ans: &requestty::Answers| {
                    ans["how_test"].as_list_items().unwrap().iter().any(|item| item.index == 0)
                })
                .default(1)
                // lol why not
                .validate_on_key(|time, _| time > 0 && time < i64::MAX)
                .validate(|time, _| {
                    if time > 0 && time < i64::MAX {
                        Ok(())
                    } else {
                        Err("Nope".to_string())
                    }
                })
                .build(),
            Question::int("temperature")
                .message("What temperature would you like to stop at? (In Celsius)")
                .when(|ans: &requestty::Answers| {
                    ans["how_test"].as_list_items().unwrap().iter().any(|item| item.text == *"Temperature")
                })
                .default(90)
                .validate_on_key(|temp, _| temp > 0 && temp < 150)
                .validate(|temp, _| {
                    let current_temp = sensors::cpu_temp(&mut sys, true);
                    if (current_temp.is_some() && current_temp.unwrap() > temp as f32) || temp >= 150 {
                        let error_message = if temp > 150 {
                            format!("Temperature must be less than 150 degrees Celsius. Current temperature is {} degrees Celsius.", current_temp.unwrap())
                        } else {
                            format!("Temperature must be greater than the current temperature of {} degrees Celsius.", current_temp.unwrap())
                        };
                        Err(error_message)
                    } else {
                        Ok(())
                    }
                })
                .build(),
            Question::select("Methods")
                .message("What method would you like to use?")
                .choices(["Fibonacci", "Primes", "Matrix Multiplication", "Float Addition", "Float Multiplication", "Square Root"])
                .build(),
        ];

        let answers = requestty::prompt(questions).unwrap();

        let chosen_index = answers.get("main_question").unwrap().as_list_item().unwrap().index as i32;
        if CPU_QUESTION_INDEXES.contains(&chosen_index) {
            let cpus = answers.get("cpu_question").unwrap().as_list_item().unwrap().index + 1;
            let duration =  answers.get("duration").unwrap().as_int().unwrap();
            let temperature = answers.get("temperature").unwrap().as_int();
            let method = answers.get("Methods").unwrap().as_list_item().unwrap().text.clone();
            let function = get_stressor_functions(method);
            let s = Duration::from_secs(duration as u64 * 60);


            do_work(
                function,
                cpus,
                temperature,
                Option::from(s) ,
                &mut sys
            );

            let answer = Question::confirm("Would you like to run another test?")
                .default(true)
                .build();
            if !requestty::prompt([answer]).unwrap().get("Would you like to run another test?").unwrap().as_bool().unwrap() {
                break;
            }
        }
    }
}

fn get_termination_options(
    sys: &mut System,
) -> Vec<String> {
    let mut options = Vec::new();
    options.push("Time".to_string());
    if sensors::cpu_temp(sys, false).is_some() {
        options.push("Temperature".to_string());
    }
    options.push("Until I say stop (Control+C)".to_string());
    options
}

fn get_stressor_functions(
    stressor: String
) -> fn() {
    match stressor.as_str() {
        "Fibonacci" => stressors::fibonacci,
        "Primes" => stressors::primes,
        "Matrix Multiplication" => stressors::matrix_multiplication,
        "Float Addition" => stressors::float_add,
        "Float Multiplication" => stressors::float_mul,
        "Square Root" => stressors::sqrt_cpu,
        _ => panic!("Invalid stressor function")
    }
}


pub fn do_work(
    function: fn(),
    thread_count: usize,
    stop_temperature: Option<i64>,
    duration: Option<Duration>,
    system: &mut System,
) {
    let start_time = Instant::now();
    let running = Arc::new(AtomicBool::new(true));
    let atomic_bool = running.clone();


    thread::scope(move |scope| {
        for _ in 0..thread_count {
            let thread_running = running.clone();
            scope.spawn(move || {
                while thread_running.load(Ordering::SeqCst) {
                    function();
                }
            });
        }

        watch_in_background(
            stop_temperature,
            duration,
            system,
            start_time,
            atomic_bool,
        );
    });
}

