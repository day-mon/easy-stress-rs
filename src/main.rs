pub mod stressors;
pub mod sensors;
mod reporting;

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
    println!("Hello {}!", sys.host_name().unwrap_or_else(|| "User".to_string()));
    println!("OS: {} v{}", sys.long_os_version().unwrap_or_else(|| "N/A".to_string()), sys.kernel_version().unwrap_or_else(|| "N/A".to_string()));
    println!("CPU: {:?} cores {:?} threads", sys.physical_core_count().unwrap_or(0),  sys.cpus().len());
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
                    let index_chosen = ans.get("main_question")
                        .expect("Main question was not found. This should not have happened")
                        .as_list_item()
                        .expect("Type of the Main question was not a ListItem.. This should not have happened!")
                        .index as i32;
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
                    ans.get("how_test")
                        .expect("The 'How would you like to terminate question' could not be found. This should not have happened!")
                        .as_list_items()
                        .expect("Type of the 'How would you like to terminate' question has been changed. This should not have happened!")
                        .iter()
                        .any(|li| li.index == 0)
                })
                .default(1)
                // lol why not
                .validate_on_key(|time, _| time > 0 && time < i64::MAX)
                .validate(|time, _| {
                    if time > 0 && time < i64::MAX {
                        Ok(())
                    } else {
                        Err("Time must be greater than 0 and less than the maximum value of an i64".into())
                    }
                })
                .build(),
            Question::int("temperature")
                .message("What temperature would you like to stop at? (In Celsius)")
                .when(|ans: &requestty::Answers| {
                    ans.get("how_test")
                        .expect("The 'How would you like to terminate question' could not be found. This should not have happened!")
                        .as_list_items()
                        .expect("Type of the 'How would you like to terminate' question has been changed. This should not have happened!")
                        .iter()
                        .any(|li| li.text == *"Temperature")
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
            .expect("Main question was not found. This should not have happened")
            .as_list_item()
            .map(|list_item| list_item.index)
            .expect("Didnt get an option for the main question") as i32;
        if CPU_QUESTION_INDEXES.contains(&chosen_index)
        {
            let duration = answers.get("duration")
                .and_then(|d| d.as_int())
                .map(|duration| Duration::from_secs(duration as u64 * 60));
            let temperature = answers.get("temperature")
                .and_then(|t| t.as_int());
            let method = answers.get("method")
                .and_then(|d| d.as_list_item())
                .map(|method| STRESSORS[method.index])
                .unwrap_or(STRESSORS[0]);

            let cpus = answers.get("cpu_question")
                .expect("CPU Option was chosen and no cpu count was given. We gotta go bye bye.")
                .as_list_item()
                .expect("Type of 'How many CPU(s) question was changed'. This should not have happened")
                .index + 1;


            match do_cpu_work(method, cpus, temperature, duration, &mut sys) {
                Ok(job) => println!("{}", job),
                Err(e) => println!("{}", e),
            }

        }

        let answer = Question::confirm("test_rerun")
            .message("Would you like to run another test?")
            .default(true)
            .build();

        let prompt = requestty::prompt([answer])
            .expect("Couldnt get the answers. Something terrible went wrong.");
        let rerun = prompt.get("test_rerun")
            .expect("Couldnt get the rerun answer. Something terrible went wrong.")
            .as_bool()
            .expect("Couldnt get the rerun answer. Something terrible went wrong.");

        if !rerun { break; }
    }
}

fn get_termination_options(
    sys: &mut System,
) -> Vec<String> {
    let mut options = Vec::new();
    options.push("Time".to_string());
    if sensors::cpu_temp(sys, false).is_some()  {
        options.push("Temperature".to_string());

    }
    options.push("Until I say stop (Control+C)".to_string());
    options
}

fn get_stressor_functions(
    stressor: &str
) -> fn() {
    match stressor {
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
    method: &str,
    cpu_count: usize,
    stop_temperature: Option<i64>,
    duration: Option<Duration>,
    system: &mut System,
)  -> Result<Job, String> {
    let start_time = Instant::now();
    let running = Arc::new(AtomicBool::new(true));

    let atomic_bool = running.clone();
    let function = get_stressor_functions(method);

    thread::scope(move |scope| {
        let mut handles = Vec::with_capacity(cpu_count);
        for _ in 0..cpu_count
        {
            let thread_running = running.clone();
            let handle = scope.spawn(move ||
                {
                    let mut iterations: u64 = 0;
                    while thread_running.load(Ordering::Relaxed)
                    {
                        function();
                        iterations += 1;
                    }
                    iterations
            });
            handles.push(handle);
        }


        let background_report = watch_in_background(
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


        Ok(
            Job {
                name: method.to_string(),
                total_iterations,
                cpu_count,
                stop_reasoning: background_report.stop_reason,
                average_cpu_temp: background_report.average_cpu_temp,
                min_cpu_temp: background_report.min_cpu_temp,
                max_cpu_temp: background_report.max_cpu_temp,
            }
        )
    })
}


pub struct Job {
    name: String,
    total_iterations: u64,
    cpu_count: usize,
    average_cpu_temp: Option<f32>,
    min_cpu_temp: Option<f32>,
    max_cpu_temp: Option<f32>,
    stop_reasoning: String
}

impl std::fmt::Display for Job {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\n📊 {} Stress Test Results 📊", self.name)?;

        let minimum_temp = match self.min_cpu_temp {
            Some(temp) => format!("🌡️ Minimum CPU Temperature: {:.2}°C", temp),
            None => String::new()
        };

        let maximum_temp = match self.max_cpu_temp {
            Some(temp) => format!("🌡️ Max CPU Temperature: {:.2}°C", temp),
            None => String::new()
        };

        let average_cpu = match self.average_cpu_temp {
            Some(temp) => format!("🌡️ Average CPU Temperature: {:.2}°C", temp),
            None => String::new()
        };




        write!(f, "\n🔥 Job Name: {} \n🔥 Total Iterations: {} \n🔥 CPU Count: {} \n⛔️ Stop Reasoning: {} \n{} \n{} \n{}",
               self.name, pretty_print_int(self.total_iterations), self.cpu_count, self.stop_reasoning, minimum_temp, maximum_temp, average_cpu)
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