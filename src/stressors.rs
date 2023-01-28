use std::fmt::{Display, Formatter};
use ocl::{Platform, Device, Context, Queue, Program, Kernel, Buffer};
use ocl::core::DeviceInfo;

#[derive(Clone)]
pub enum Stressor {
    Fibonacci,
    Primes,
    MatrixMultiplication,
    FloatAddition,
    FloatMultiplication,
    SquareRoot
}

impl Display for Stressor {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Stressor::Fibonacci => f.write_str("Fibonacci"),
            Stressor::Primes => f.write_str( "Primes"),
            Stressor::MatrixMultiplication => f.write_str("Matrix Multiplication"),
            Stressor::FloatAddition => f.write_str( "Float Addition"),
            Stressor::FloatMultiplication => f.write_str("Float Multiplication"),
            Stressor::SquareRoot => f.write_str("Square Root"),
        }
    }
}

pub const OPENCL_VECTOR_SIZE: usize = 10_000;

pub const OPENCL_FLOAT_ADD: &str = r#"
    __kernel void float_add(__global float* a, __global float* b) {
        int id = get_global_id(0);
        b[id] = a[id] + 0.1391273;
    }
"#;

pub const OPENCL_MATRIX_MULTIPLICATION: &str = r#"
__kernel void matrix_mult(__global float* a, __global float* b, __global float* c) {
    int id = get_global_id(0);
    c[id] = a[id] * b[id];
}
"#;

pub const OPENCL_SQUARE_ROOT: &str = r#"
__kernel void sqrt(__global float* a, __global float* b) {
    int id = get_global_id(0);
    b[id] = sqrt(a[id]);
}
"#;

pub const OPENCL_FIBONACCI: &str = r#"
__kernel void fibonacci(__global int* a, __global int* b) {
    int id = get_global_id(0);
    b[id] = a[id] + a[id + 1];
}
"#;

pub const OPENCL_FACTORIAL: &str = r#"
__kernel void factorial(__global int* a, __global int* b) {
    int id = get_global_id(0);
    b[id] = a[id] * a[id + 1];
}
"#;

pub const OPENCL_PRIMES: &str = r#"
__kernel void primes(__global int* a, __global int* b) {
    int id = get_global_id(0);
    b[id] = a[id] * a[id + 1];
}
"#;




pub fn sqrt_cpu() {
    let _ = (952.0_f32).sqrt();
}

pub fn factorial_cpu() {
    let mut _factorial = 1;
    for i in 1..=100 {
        _factorial *= i;
    }
}


pub fn fibonacci_cpu() {
    let mut a: u64 = 0;
    let mut b = 1;
    for _ in 0..50 {
        let c = a + b;
        a = b;
        b = c;
    }
}


pub fn float_add() {
    let mut _x = 0.0;
    for _ in 0..10 {
        _x += 0.0000001;
    }
}

pub fn primes() {
    let mut _primes = 0;
    for i in 2..100000 {
        if is_prime(i) {
            _primes += 1;
        }
    }
}

fn is_prime(n: i32) -> bool {
    if n == 2 {
        return true;
    }
    if n % 2 == 0 {
        return false;
    }
    let mut i = 3;
    while i * i <= n {
        if n % i == 0 {
            return false;
        }
        i += 2;
    }
    true
}

pub fn matrix_multiplication() {
    let mut matrix = [[0.0; 100]; 100];
    for (i, row) in matrix.iter_mut().enumerate() {
        for (j, col) in row.iter_mut().enumerate().take(100) {
            *col = (i * j) as f32;
        }
    }
    for _ in 0..100 {
        let mut result = [[0.0; 100]; 100];
        for i in 0..100 {
            for j in 0..100 {
                for k in 0..100 {
                    result[i][j] += matrix[i][k] * matrix[k][j];
                }
            }
        }
    }
}

pub fn float_mul() {
    let mut _x = 1.0;
    for _ in 0..10 {
        _x *= 1.0000001;
    }
}

pub struct OpenCLContext {
    pub platform: Platform,
    pub device: Device,
    pub context: Context,
    pub queue: Queue,
}

impl OpenCLContext {
    pub fn new(device: Device) -> Result<Self, String> {
        let platform = Platform::default();
        let context = Context::builder()
            .platform(platform)
            .devices(device)
            .build()?;
        let queue = Queue::new(&context, device, None)?;
        Ok(OpenCLContext {
            platform,
            device,
            context,
            queue,
        })
    }
}



pub struct OpenCLProgram {
    pub program: Program,
    pub kernel: Kernel,
    pub wg_size: Vec<usize>,
}


impl OpenCLProgram {
    pub fn new(context: &OpenCLContext, source: &str, kernel_name: &str, kernel_args: &[[f32; OPENCL_VECTOR_SIZE]]) -> Result<Self, String> {
        let program = Program::builder()
            .src(source)
            .devices(context.device)
            .build(&context.context)?;
        let wg_size = context.device.info(DeviceInfo::MaxWorkItemSizes)?.to_string();
        let wg_size = wg_size.replace(['[', ']'], "");
        let wg_size: Vec<usize> = wg_size.split(',').map(|s| s.trim().parse().unwrap()).collect();

        let kernel = if kernel_args.len() == 2 {
            Kernel::builder()
                .name(kernel_name)
                .program(&program)
                .queue(context.queue.clone())
                .arg(None::<&Buffer<f32>>) // Placeholder for the first argument
                .arg(None::<&Buffer<f32>>) // Placeholder for the second argument
                .build()?
        } else {
            Kernel::builder()
                .name(kernel_name)
                .program(&program)
                .queue(context.queue.clone())
                .arg(None::<&Buffer<f32>>) // Placeholder for the first argument
                .arg(None::<&Buffer<f32>>) // Placeholder for the second argument
                .arg(None::<&Buffer<f32>>) // Placeholder for the third argument
                .build()?
        };

        for (i, arg) in kernel_args.iter().enumerate()
        {
            let buffer = Buffer::<f32>::builder()
                .queue(context.queue.clone())
                .flags(ocl::flags::MEM_READ_WRITE)
                .copy_host_slice(arg)
                .len(OPENCL_VECTOR_SIZE)
                .build()?;
            kernel.set_arg(i, buffer)?;
        }




        Ok(OpenCLProgram { program, kernel, wg_size })
    }

    pub fn run(&self) -> ocl::Result<()> {
        unsafe {
            // spawn the kernel on every compute unit
            self.kernel
                .cmd()
                .global_work_size((self.wg_size[0], self.wg_size[1], self.wg_size[2]))
                .enq()
        }
    }
}
