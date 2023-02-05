extern crate core;

pub mod stressors;
pub mod sensors;
mod reporting;
mod components;
mod prompt;

use std::io::{stdout, Write};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize};
use std::sync::atomic::Ordering::Relaxed;
use std::{panic, thread};
use std::any::Any;
use std::time::{Duration, Instant};
use colored::Colorize;
use inquire::{Confirm, CustomType, MultiSelect, Select};
use inquire::error::InquireResult;

use ocl::{Device, DeviceType, Platform};
use ocl::core::DeviceInfo;
use sysinfo::{System, SystemExt};
use crate::components::GreetingValues;
use crate::reporting::{prettify_output, watch_in_background};
use crate::stressors::{OPENCL_FLOAT_ADD, OPENCL_MATRIX_MULTIPLICATION, OPENCL_SQUARE_ROOT, OPENCL_VECTOR_SIZE, OpenCLContext, OpenCLProgram, Stressor};

const NO_OPENCL_STRING: &str = r#"No OpenCL platforms found. This is probably because you dont have a GPU or you dont have GPU compatible drivers installed.
If you have a GPU and the drivers are installed, please report this issue to the developers.

Intel GPUs:
 - https://www.intel.com/content/www/us/en/search.html
AMD GPUs:
 - https://www.amd.com/en/support
Nvidia GPUs:
 - https://www.nvidia.com/Download/index.aspx?lang=en-us

You can report this issue here: https://github.com/day-mon/easy-stress-rs"#;



fn main() -> InquireResult<()> {
    println!("Looking for GPU Platforms...");
    let platforms = setup();

    let platform = match platforms {
        Ok(platforms) => obtain_platform(platforms),
        Err(_) => {
            println!("{NO_OPENCL_STRING}");
            None
        }
    };

    println!("\rGrabbing System Information...");
    let mut sys = System::new_all();
    let mut gpu_ctx: Option<OpenCLContext> = None;
    let gpu_device: Option<Device> = None;

    let system_information = GreetingValues::new(&sys, platform);
    println!("{system_information}");

    loop {
        let main_question= Select::new("What would you like to stress?", get_stressed_components(&system_information))
            .prompt()?;

        let gpu_question = match main_question {
            "GPU" => Select::new("What GPU would you like to use", system_information.get_gpus_str())
                .prompt()
                .ok(),
            _ => None
        };

        let cpu_questions = match main_question {
            "CPU" => Select::new("How many CPU(s) would you like to use", system_information.get_cpus_str())
                .with_formatter(&|i| format!("{} CPU(s)", i.index + 1))
                .prompt()
                .ok(),
            _ => None
        };


        let termination_method = MultiSelect::new("How would you like the test to terminate? (Will terminate when any condition is met in this order)", get_termination_options(&mut sys, main_question))
            .with_validator(prompt::termination_method_validator)
            .with_keep_filter(false)
            .prompt()?;


        let temperature = match termination_method.iter().any(|&i| i == "Temperature") {
            true => {
                let current_temperature = sensors::cpu_temp(&mut sys, true);
                CustomType::<u8>::new("What temperature would you like to stop at? (In Celsius)")
                    .with_default(90)
                    .with_validator(move |input: &u8| prompt::temperature_validator(input, current_temperature))
                    .with_help_message("Please pick a number 1 -> 255 (Even though if your computer gets this hot something is either wrong or you're crazy, it is recommended to not go above 90C)")
                    .with_error_message("Please type a valid number")
                    .prompt()
                    .ok()
            }
            false => None
        };

        let duration = match termination_method.iter().any(|&i| i == "Time") {
            true => CustomType::<u16>::new("How long would you like the stress test to last? (in minutes)")
                .with_default(1)
                .with_validator(prompt::duration_validator)
                .with_help_message("Type in a number between 1 -> 65536")
                .with_error_message("This number is too big. Number has to be in the range 1 -> 65536.")
                .prompt()
                .ok(),
            false => None
        };
        let method = Select::new("What method would you like to use?", get_stressors(main_question))
            .prompt()?;

        let duration = duration.map(|dur| Duration::from_secs(dur as u64 * 60));

        if main_question == "CPU"
        {
            let cpus = cpu_questions
                .expect("CPU Option was chosen and no cpu count was given. We gotta go bye bye.");
            match do_cpu_work(method, cpus, temperature, duration, &mut sys) {
                Ok(job) => println!("{job}"),
                Err(e) => println!("{e}"),
            }
        }
        else if main_question == "GPU"
        {
            let gpu_text = gpu_question.expect("GPU Option was chosen and no gpu was given. We gotta go bye bye.");
            let device = gpu_device.unwrap_or_else(|| Device::from(*{
                let gpu_options = get_gpu_options().expect("Couldn't get GPU options. Something went wrong.");

                gpu_options
                    .into_iter()
                    .find(|&gpu| {
                        match gpu.info(DeviceInfo::Name) {
                            Ok(name) => name.to_string() == *gpu_text,
                            Err(_) => false,
                        }
                    })
                    .expect("Couldn't find GPU device. Something went wrong.")
            }));

            if gpu_ctx.is_none() {
                gpu_ctx = OpenCLContext::new(device)
                    .ok();
            }
            let output = gpu_ctx.as_ref().and_then(|ctx| {
                get_opencl_program(&method, ctx)
                    .and_then(|program| do_gpu_work(program, duration, method))
                    .map(|job| format!("{job}"))
                    .ok()
            }).unwrap_or_else(|| "Could not get GPU context. Something went wrong.".to_string());

            println!("{output}");
        }


        let rerun_question = Confirm::new("Would you like to run another test?")
            .with_help_message("To continue type (y) to exit type (n)")
            .prompt()?;

        if !rerun_question { break; }
    }

    Ok(())
}

fn get_stressed_components(sys_info: &GreetingValues) -> Vec<&str> {
    if sys_info.gpu_information.is_empty() {
        vec!["CPU"]
    } else {
        vec!["CPU", "GPU"]
    }
}

fn get_stressors(
    choice: &str
) -> Vec<Stressor> {
    match choice {
        "CPU" => if cfg!(target_arch = "x86_64") {
            vec![Stressor::Fibonacci, Stressor::FloatAddition, Stressor::FloatMultiplication, Stressor::MatrixMultiplication, Stressor::SquareRoot, Stressor::Primes, Stressor::InverseSquareRoot, Stressor::FloatDivision]
        } else {
            vec![Stressor::Fibonacci, Stressor::FloatAddition, Stressor::FloatMultiplication, Stressor::MatrixMultiplication, Stressor::SquareRoot, Stressor::Primes, Stressor::FloatDivision]
        },
        "GPU" => vec![Stressor::SquareRoot, Stressor::MatrixMultiplication, Stressor::FloatAddition],
        _ => panic!("Invalid stressor")
    }
}

fn get_termination_options(sys: &mut System, chosen_component: &str) -> Vec<&'static str> {
    if let ("CPU", true) = (chosen_component, sensors::cpu_temp(sys, true).is_some()) {
        vec!["Time", "Temperature"]
    } else {
        vec!["Time"]
    }
}

fn get_stressor_functions(
    stressor: &Stressor
) -> fn() {
    match stressor {
        Stressor::Fibonacci => stressors::fibonacci_cpu,
        Stressor::Primes => stressors::primes,
        Stressor::MatrixMultiplication => stressors::matrix_multiplication,
        Stressor::FloatAddition => stressors::float_add,
        Stressor::FloatMultiplication => stressors::float_mul,
        Stressor::SquareRoot => || { stressors::sqrt_cpu(std::hint::black_box(1_143_243_423.112_354_3)) },
        Stressor::FloatDivision => stressors::float_division,
        Stressor::InverseSquareRoot => if cfg!(target_arch = "x86_64") {
            || { stressors::invsqrt(std::hint::black_box(1_143_243_423.112_354_3)) }
        } else {
            panic!("Invalid stressor")
        }
    }
}

pub fn get_opencl_program(
    method: &Stressor,
    ctx: &OpenCLContext,
) -> Result<OpenCLProgram, String> {
    match method {
        Stressor::SquareRoot => {
            // yeah, lets spam sqrt 952 on gpu
            let sqrt_vector = vec![952_f32; OPENCL_VECTOR_SIZE];
            let result_vector = vec![0_f32; OPENCL_VECTOR_SIZE];
            OpenCLProgram::new(ctx, OPENCL_SQUARE_ROOT, "sqrt", vec![sqrt_vector, result_vector])
        }
        Stressor::FloatAddition => {
            let f_add_vector = vec![952.139_1_f32; OPENCL_VECTOR_SIZE];
            let result_vector = vec![0_f32; OPENCL_VECTOR_SIZE];
            OpenCLProgram::new(ctx, OPENCL_FLOAT_ADD, "float_add", vec![f_add_vector, result_vector])
        }
        Stressor::MatrixMultiplication => {
            let matrix_a = vec![201.139_13_f32; OPENCL_VECTOR_SIZE];
            let matrix_b = vec![952.139_1_f32; OPENCL_VECTOR_SIZE];
            let result_vector = vec![0_f32; OPENCL_VECTOR_SIZE];
            OpenCLProgram::new(ctx, OPENCL_MATRIX_MULTIPLICATION, "matrix_mult", vec![matrix_a, matrix_b, result_vector])
        }
        _ => {
            println!("No method found, defaulting to sqrt");
            let sqrt_vector = vec![952_f32; OPENCL_VECTOR_SIZE];
            let result_vector = vec![0_f32; OPENCL_VECTOR_SIZE];
            OpenCLProgram::new(ctx, OPENCL_SQUARE_ROOT, "sqrt", vec![sqrt_vector, result_vector])
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
        let mut iter_failed = false;

        program.run().unwrap_or_else(|_| {
            println!("Error occurred while attempting to enqueue the kernel. If this continues to happen just Control+C");
            iter_failed = true;
        });

        if iter_failed { continue; }

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
    stop_temperature: Option<u8>,
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
                    // for the stressor functions check the asm
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

pub fn setup() -> Result<Vec<Platform>, Box<dyn Any + Send + 'static>> {
    inquire::set_global_render_config(prompt::get_nice_render_config_new());
    let normal_hook = panic::take_hook();
    panic::set_hook(Box::new(move |_info| {}));
    let platforms = panic::catch_unwind(Platform::list);
    panic::set_hook(normal_hook);
    platforms
}

pub struct Job {
    name: String,
    total_iterations: u64,
    cpu_count: Option<usize>,
    average_cpu_temp: Option<f32>,
    min_cpu_temp: Option<f32>,
    max_cpu_temp: Option<f32>,
    stop_reasoning: String,
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
            write!(f, "\n⇁ Peak CPU Temperature: {max_temp:.2}°C")?;
        }

        if let Some(min_temp) = self.min_cpu_temp {
            write!(f, "\n⇁ Minimum CPU Temperature: {min_temp:.2}°C")?;
        }

        if let Some(average_temp) = self.average_cpu_temp {
            write!(f, "\n⇁ Average CPU Temperature: {average_temp:.2}°C")?;
        }


        Ok(())
    }
}
fn obtain_platform(platforms: Vec<Platform>) -> Option<Platform> {
    match platforms.len() {
        0 => {
            println!("{NO_OPENCL_STRING}");
            None
        },
        1 => Some(platforms[0]),
        _ => match Select::new("Which GPU Platform would you like to use", platforms).with_formatter(&prompt::platform_formatter).prompt() {
            Ok(platform) => Some(platform),
            Err(error) => {
                eprintln!("Some error has occurred. GPU Stress testing will be disabled. Error {error}");
                None
            }
        }
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
