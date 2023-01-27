use std::ops::Add;
use ocl::{Platform, Device, Context, Queue, Program, Kernel, Buffer};

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
    for i in 0..100 {
        for j in 0..100 {
            matrix[i][j] = (i * j) as f32;
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
}

impl OpenCLProgram {
    pub fn new(context: &OpenCLContext, source: &str, kernel_name: &str, kernel_args: &[[f32; 1000]]) -> Result<Self, String> {
        let program = Program::builder()
            .src(source)
            .devices(context.device)
            .build(&context.context).unwrap();
        let kernel = Kernel::builder()
            .name(kernel_name)
            .program(&program)
            .queue(context.queue.clone())
            .arg(None::<&Buffer<f32>>) // Placeholder for the first argument
            .arg(None::<&Buffer<f32>>) // Placeholder for the second argument
            .build().unwrap();

        for (i, arg) in kernel_args.iter().enumerate() {
            let buffer = Buffer::<f32>::builder()
                .queue(context.queue.clone())
                .flags(ocl::flags::MEM_READ_WRITE)
                .copy_host_slice(arg)
                .len(1000)
                .build()?;
            kernel.set_arg(i, buffer)?;
        }

        // Set kernel arguments


        Ok(OpenCLProgram { program, kernel })
    }

    pub fn run(&self) {
        unsafe {
            self.kernel.
                cmd()
                .global_work_size(10000)
                .enq().unwrap()

        }
    }
}
