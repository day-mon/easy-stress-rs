pub mod stressors;
pub mod sensors;
mod reporting;
mod components;

use std::io::{stdout, Write};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize};
use std::sync::atomic::Ordering::Relaxed;
use std::thread;
use std::time::{Duration, Instant};
use colored::Colorize;
use ocl::{Device, DeviceType, Platform};
use ocl::core::DeviceInfo;
use requestty::{Question};
use sysinfo::{System, SystemExt};
use crate::components::GreetingValues;
use crate::reporting::{prettify_output, watch_in_background};
use crate::stressors::*;


fn main() {
    println!("Grabbing System Information...");
    let mut sys = System::new_all();
    let mut gpu_ctx: Option<OpenCLContext> = None;
    let gpu_device: Option<Device> = None;

    let system_information = GreetingValues::new(&sys);
    println!("{system_information}");

    loop {
        let questions = [
            Question::select("main_question")
                .message("What would you like to test?")
                .choices(get_stressed_components(&system_information))
                .build(),
            Question::select("gpu_select")
                .message("What GPU would you like to use")
                .choices(system_information.gpu_information.iter().map(|item| item.name.clone()).collect::<Vec<String>>())
                .when(|ans: &requestty::Answers|
                    ans.get("main_question")
                        .expect("Main question was not found. This should not have happened")
                        .as_list_item()
                        .expect("Type of the Main question was not a ListItem.. This should not have happened!")
                        .index as i32 == 1
                )
                .build(),
            Question::select("cpu_question")
                .message("How many CPU(s) would you like to use?")
                .choices((1..=sys.cpus().len()).map(|cpu| format!("{cpu} CPU(s)")).collect::<Vec<String>>())
                .when(|ans: &requestty::Answers|
                    ans.get("main_question")
                        .expect("Main question was not found. This should not have happened")
                        .as_list_item()
                        .expect("Type of the Main question was not a ListItem.. This should not have happened!")
                        .index as i32 == 0
                )
                .build(),
        ];

        let answers = requestty::prompt(questions)
            .expect("Couldnt get the answers. Something went terrible wrong.");
        let chosen_index = answers.get("main_question")
            .expect("Main question was not found. This should not have happened")
            .as_list_item()
            .map(|list_item| list_item.index)
            .expect("Didnt get an option for the main question");
        let stressor_choices = get_stressors(&chosen_index);

        let questions_part_two = [
        Question::select("method")
            .message("What method would you like to use?")
            .choices(stressor_choices.iter().map(|item| item.to_string()))
            .build(),
         Question::multi_select("how_test")
            .message("How would you like the test to terminate? (Will terminate when any condition is met in this order)")
            .choices(get_termination_options(&mut sys, chosen_index))
            .validate(|ans, _ | {
                if ans.iter().filter(|&&a| a).count() < 1 {
                    Err("You must choose at least one option.".into())
                } else {
                    Ok(())
                }
            }).build(),
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
                    match (current_temp, temp) {
                        (Some(current), 0..=150) => Err(format!("Temperature must be less than 150 degrees Celsius. Current temperature is {current} degrees Celsius.")),
                        (Some(current), _) => Err(format!("Temperature must be greater than the current temperature of {current} degrees Celsius.")),
                        (None, _) => Ok(())
                    }
                }).build(),
        ];



        let answers_two = requestty::prompt(questions_part_two)
            .expect("Could not get method.. :(");
        let duration = answers_two.get("duration")
            .and_then(|d| d.as_int())
            .map(|duration| Duration::from_secs(duration as u64 * 60));
        let temperature = answers_two.get("temperature")
            .and_then(|t| t.as_int());
        let method = answers_two.get("method")
            .and_then(|opt| opt.as_list_item())
            .map(|method| stressor_choices[method.index].clone())
            .unwrap_or(stressor_choices[0].clone());

        if chosen_index == 0
        {
            let cpus = &answers.get("cpu_question")
                .expect("CPU Option was chosen and no cpu count was given. We gotta go bye bye.")
                .as_list_item()
                .expect("Type of 'How many CPU(s) question was changed'. This should not have happened")
                .index + 1;

            match do_cpu_work(method, cpus, temperature, duration, &mut sys) {
                Ok(job) => println!("{job}"),
                Err(e) => println!("{e}"),
            }
        }
        else if chosen_index == 1
        {
            let gpu_text = &answers.get("gpu_select")
                .expect("GPU Option was chosen and no gpu name was given. We gotta go bye bye.")
                .as_list_item()
                .expect("Type of 'Which GPU question was changed'. This should not have happened")
                .text;


            let device = gpu_device.unwrap_or_else(|| {
                let gpu_options = match get_gpu_options() {
                    Ok(devices) => devices,
                    Err(error) => panic!("Couldnt get gpu devices. Error: {error}")
                };
                let Some(gpu) = gpu_options.iter().find(|&gpu| {
                    let g =  match gpu.info(DeviceInfo::Name) {
                        Ok(name) => name,
                        Err(_) => return false,
                    }.to_string();
                    g == *gpu_text
                }) else {
                    panic!("Couldnt find gpu device. Something went wrong.");
                };
                *gpu
            });

            if gpu_ctx.is_none() {
                gpu_ctx = match OpenCLContext::new(device) {
                    Ok(ctx) => Some(ctx),
                    Err(error) => {
                        println!("Could not get GPU context. Something went wrong. Error {error}");
                        None
                    }
                }
            }

            // going to fix this later
            let output = match &gpu_ctx {
                Some(ctx) => {
                    match get_opencl_program(&method, ctx) {
                        Ok(program) => {
                            match do_gpu_work(program, duration, method) {
                                Ok(job) => format!("{job}"),
                                Err(e) => e.to_string(),
                            }
                        },
                        Err(err) => {
                            err
                        }
                    }

                },
                None => "Could not get GPU context. Something went wrong.".to_string()
            };

            println!("{output}");
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

fn get_stressed_components(sys_info: &GreetingValues) -> Vec<String> {
    let mut ans: Vec<String> = Vec::with_capacity(2);
    ans.push("CPU".to_string());
    if !sys_info.gpu_information.is_empty() {
        ans.push("GPU".to_string())
    }
    ans

}

fn get_stressors(
    index: &usize
) -> Vec<Stressor> {

    match index {
        0 => {
            let mut cpu_options = Vec::with_capacity(6);
            cpu_options.extend_from_slice(
                &[Stressor::Fibonacci, Stressor::FloatAddition, Stressor::FloatMultiplication, Stressor::MatrixMultiplication,
                Stressor::SquareRoot, Stressor::Primes]
            );
            cpu_options

        },
        1 => {
            let mut gpu_options = Vec::with_capacity(3);
            gpu_options.extend_from_slice(
                &[Stressor::SquareRoot, Stressor::MatrixMultiplication, Stressor::FloatAddition]
            );
            gpu_options
        }
        _ => panic!("Invalid stressor")
    }
}

fn get_termination_options(sys: &mut System, chosen_index: usize) -> Vec<String> {
    let mut options = Vec::new();
    options.push("Time".to_string());
    if let (0, true) = (chosen_index, sensors::cpu_temp(sys, true).is_some()) {
        options.push("Temperature".to_string())
    }
    options
}

fn get_stressor_functions(
    stressor: &Stressor
) -> fn() {
    match stressor {
        Stressor::Fibonacci => fibonacci_cpu,
        Stressor::Primes => primes,
        Stressor::MatrixMultiplication => matrix_multiplication,
        Stressor::FloatAddition => float_add,
        Stressor::FloatMultiplication => float_mul,
        Stressor::SquareRoot => sqrt_cpu,
    }
}

pub fn get_opencl_program(
    method: &Stressor,
    ctx: &OpenCLContext,
) -> Result<OpenCLProgram, String> {

    match method {
        Stressor::SquareRoot => {
            // yeah, lets spam sqrt 952 on gpu
            let sqrt_vector = [952_f32; OPENCL_VECTOR_SIZE];
            let result_vector = [0_f32; OPENCL_VECTOR_SIZE];
            OpenCLProgram::new(ctx, OPENCL_SQUARE_ROOT, "sqrt", &[sqrt_vector, result_vector])
        },
        Stressor::FloatAddition  => {
            let f_add_vector = [952.139_1_f32; OPENCL_VECTOR_SIZE];
            let result_vector = [0_f32; OPENCL_VECTOR_SIZE];
            OpenCLProgram::new(ctx, OPENCL_FLOAT_ADD, "float_add", &[f_add_vector, result_vector])
        },
        Stressor::MatrixMultiplication => {
            let matrix_a = [201.139_13_f32; OPENCL_VECTOR_SIZE];
            let matrix_b = [952.139_1_f32; OPENCL_VECTOR_SIZE];
            let result_vector = [0_f32; OPENCL_VECTOR_SIZE];
            OpenCLProgram::new(ctx, OPENCL_MATRIX_MULTIPLICATION, "matrix_mult", &[matrix_a, matrix_b, result_vector] )
        },
        _ => {
            println!("No method found, defaulting to sqrt");
            let sqrt_vector = [952_f32; OPENCL_VECTOR_SIZE];
            let result_vector = [0_f32; OPENCL_VECTOR_SIZE];
            OpenCLProgram::new(ctx, OPENCL_SQUARE_ROOT, "sqrt", &[sqrt_vector, result_vector])
        }
    }
}

fn get_gpu_options() -> ocl::Result<Vec<Device>> {
    let platform = Platform::default();
    Device::list(platform, Some(DeviceType::GPU))
}

fn do_gpu_work(
    program: OpenCLProgram,
    duration: Option<Duration>,
    method: Stressor,
) -> Result<Job, String> {
    println!("{}", format!("🏁 Starting {method}. If you wish to stop the test at any point hold Control+C").white().bold());
    let start_time = Instant::now();
    let mut iterations = 0;

    program.run()
        .map_err(|error| format!("Some error has occurred while trying to do a test run to see if {method} runs on your computer. Error: {error}"))?;


    loop {
        if let Some(duration) = duration {
            if start_time.elapsed() > duration {
                break;
            }
        }

        // ignore error
        program.run().unwrap_or_else(|_| {
            println!("Error occurred while attempting to enqueue the kernel. If this continues to happen just Control+C")
        });
        iterations += 1;

        let output = prettify_output(duration, start_time, None);
        print!("{output}");
        let _ = stdout().flush();
    }

    Ok(
            Job {
            name: method.to_string(),
            total_iterations: iterations,
            cpu_count: None,
            average_cpu_temp: None,
            min_cpu_temp: None,
            max_cpu_temp: None,

            stop_reasoning: "Time limit exceeded".to_string(),
        }
    )
}


fn do_cpu_work(
    method: Stressor,
    cpu_count: usize,
    stop_temperature: Option<i64>,
    duration: Option<Duration>,
    system: &mut System,
) -> Result<Job, String> {
    let start_time = Instant::now();
    let running = Arc::new(AtomicUsize::new(0));

    let atomic_bool = running.clone();
    let function = get_stressor_functions(&method);

    println!("{}", format!("🏁 Starting {method}. If you wish to stop the test at any point hold Control+C").white().bold());


    thread::scope(move |scope| {
        let mut handles = Vec::with_capacity(cpu_count);
        for _ in 0..cpu_count
        {
            let thread_running = running.clone();
            let handle = scope.spawn(move ||
                {
                    let mut iterations: u64 = 0;
                    while thread_running.load(Relaxed) == 0
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

        let stop_reason = match running.load(Relaxed) {
            1 => "Time Limit exceeded",
            2 => "Temperature exceeded",
            3 => "Ctrl-C caught",
            _ => panic!("This should have never happened. {} is not a valid option", running.load(Relaxed))
        }.to_string();

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
                cpu_count: Some(cpu_count),
                stop_reasoning: stop_reason,
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
    cpu_count: Option<usize>,
    average_cpu_temp: Option<f32>,
    min_cpu_temp: Option<f32>,
    max_cpu_temp: Option<f32>,
    stop_reasoning: String
}

impl std::fmt::Display for Job {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\n{} Stress Test Results ", self.name)?;

        write!(f, "\n⇁ Job Name: {} \n⇁ Total Iterations: {} \n⇁ Stop Reasoning: {}",
               self.name, pretty_print_int(self.total_iterations), self.stop_reasoning)?;

        if let Some(cpus) = self.cpu_count {
            write!(f, "\n⇁ CPU Count: {cpus}")?;
        }

        if let Some(max_temp) = self.max_cpu_temp {
            write!(f, "\n⇁ Maximum CPU Temperature: {max_temp:.2}°C")?;
        }

        if let Some(min_temp) = self.min_cpu_temp {
            write!(f, "\n⇁ Max CPU Temperature: {min_temp:.2}°C")?;
        }

        if let Some(average_temp) = self.average_cpu_temp {
            write!(f, "\n⇁ Max CPU Temperature: {average_temp:.2}°C")?;
        }


        Ok(())
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