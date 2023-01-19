pub mod stressors;
pub mod sensors;
mod reporting;


use std::fmt::write;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};
use requestty::{Question};
use sysinfo::{System, SystemExt};
use crate::reporting::watch_in_background;

fn main() {
    let mut sys = System::new_all();

    // Display system information:s
    println!("Hello {}!", sys.host_name().unwrap());
    println!("OS: {} v{}", sys.long_os_version().unwrap(), sys.kernel_version().unwrap());
    println!("CPU: {:?} cores {:?} threads", sys.physical_core_count().unwrap(),  sys.cpus().len());
    println!("Memory: {:?} GBs", sys.total_memory() / 1024 / 1024 / 1024);

    const CPU_QUESTION_INDEXES: [i32; 3] = [0, 3, 4];
    const STRESSORS: [&str; 6] = ["Fibonacci", "Primes", "Matrix Multiplication", "Float Addition", "Float Multiplication", "Square Root"];

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
            Question::select("method")
                .message("What method would you like to use?")
                .choices(STRESSORS)
                .build(),
        ];

        let answers = requestty::prompt(questions)
            .expect("Couldnt get the answers. Something went terrible wrong.");

        let chosen_index = answers.get("main_question")
            .and_then(|opt| opt.as_list_item())
            .map(|list_item| list_item.index)
            .expect("Didnt get an option for the main question") as i32;
        if CPU_QUESTION_INDEXES.contains(&chosen_index) {
            let duration = answers.get("duration").and_then(|d| d.as_int()).map(|duration| Duration::from_secs(duration as u64 * 60));
            let temperature = answers.get("temperature").and_then(|t| t.as_int());
            let method = answers.get("method")
                .and_then(|d| d.as_list_item())
                .map(|method| STRESSORS[method.index])
                .unwrap_or(STRESSORS[0])
                .to_string();
            let cpus = answers.get("cpu_question")
                .and_then(|item| item.as_list_item())
                .map(|list_item| list_item.index + 1)
                .expect("CPU Option was chosen and no cpu count was given. We gotta go bye bye.");


            let job = do_cpu_work(method, cpus, temperature, duration, &mut sys);

            if job.is_ok() {
                println!("{}", job.unwrap())
            } else {
                eprintln!("{}", job.err().unwrap())
            }


            let answer = Question::confirm("test_rerun")
                .message("Would you like to run another test?")
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


pub fn do_cpu_work(
    method: String,
    thread_count: usize,
    stop_temperature: Option<i64>,
    duration: Option<Duration>,
    system: &mut System,
)  -> Result<Job, String> {
    let start_time = Instant::now();
    let running = Arc::new(AtomicBool::new(true));
    let atomic_bool = running.clone();
    let function = get_stressor_functions(method.clone());

    thread::scope(move |scope| {
        let mut handles = Vec::with_capacity(thread_count);
        for _ in 0..thread_count {
            let thread_running = running.clone();
            let handle = scope.spawn(move || {
                let mut iterations: u64 = 0;
                while thread_running.load(Ordering::SeqCst) {
                    function();
                    iterations += 1;
                }
                iterations
            });
            handles.push(handle);
        }

        let (temp, stop_reason) = watch_in_background(
            stop_temperature,
            duration,
            system,
            start_time,
            atomic_bool,
        );

        let mut total_iterations = 0;
        for handle in handles {
            if let Ok(iterations) = handle.join() {
                total_iterations += iterations;
            } else {
                return Err("Failed to join thread".to_string());
            }
        }


        Ok(Job {
            name: method,
            total_iterations,
            thread_count,
            stop_reasoning: stop_reason,
            average_cpu_temp: temp
        })
    })
}


pub struct Job {
    name: String,
    total_iterations: u64,
    thread_count: usize,
    average_cpu_temp: Option<f32>,
    stop_reasoning: String
}

impl std::fmt::Display for Job {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\n🔥 Job Name: {} \n🔥 Total Iterations: {} \n🔥 Thread Count: {} \n⛔️ Stop Reasoning: {} \n🌡️ Average CPU Temperature: {}\n",
               self.name, pretty_print_int(self.total_iterations), self.thread_count, self.stop_reasoning, self.average_cpu_temp.unwrap())


    }
}

fn pretty_print_int(i: u64) -> String {
    let mut s = String::new();
    let i_str = i.to_string();
    let a = i_str.chars().rev().enumerate();
    for (idx, val) in a {
        if idx != 0 && idx % 3 == 0 {
            s.insert(0, ',');
        }
        s.insert(0, val);
    }
    s
}