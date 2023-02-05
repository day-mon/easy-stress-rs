use std::fmt::{Display, Formatter};
use ocl::{Platform, Device, Context, Queue, Program, Kernel, Buffer};
use ocl::core::{DeviceInfo, DeviceInfoResult};


#[derive(Clone)]
pub enum Stressor {
    Fibonacci,
    Primes,
    MatrixMultiplication,
    FloatAddition,
    FloatMultiplication,
    FloatDivision,
    SquareRoot,
    InverseSquareRoot
}

impl Display for Stressor {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Stressor::Fibonacci => f.write_str("Fibonacci"),
            Stressor::Primes => f.write_str("Primes"),
            Stressor::MatrixMultiplication => f.write_str("Matrix Multiplication"),
            Stressor::FloatAddition => f.write_str("Float Addition"),
            Stressor::FloatMultiplication => f.write_str("Float Multiplication"),
            Stressor::FloatDivision => f.write_str("Float Division"),
            Stressor::SquareRoot => f.write_str("Square Root"),
            Stressor::InverseSquareRoot => f.write_str("Inverse Square Root")
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

/*
sqrt_cpu:
        push    rax
        mov     eax, 10000000
        mov     rcx, rsp
.LBB0_1:
        movsd   qword ptr [rsp], xmm0
        movsd   xmm1, qword ptr [rsp]
        sqrtsd  xmm1, xmm1
        movsd   qword ptr [rsp], xmm1
        dec     eax
        jne     .LBB0_1
        pop     rax
        ret
 */
pub fn sqrt_cpu(num: f64)  {
    for _ in 0..10_000_000 {
        std::hint::black_box(std::hint::black_box(num).sqrt());
    }
}

/*
example::invsqrt:
        push    rax
        mov     eax, 10000000
.LBB0_1:

        rsqrtss xmm0, xmm0

        dec     eax
        jne     .LBB0_1
        pop     rax
        ret
 */
pub fn invsqrt(mut x: f32)  {
    for _ in 0..10_000_000 {
        unsafe {
            std::arch::asm!("rsqrtss {x}, {x}", x = inout(xmm_reg) x);
        }
    }
}
/*
example::factorial_cpu:
        push    r14
        push    rbx
        sub     rsp, 16
        cmp     rdi, 2
        mov     rax, rsi
        sbb     rax, 0
        jb      .LBB0_3
        mov     ecx, 1
        xor     r9d, r9d
        mov     r8, rsp
        mov     ebx, 1
        xor     r14d, r14d
        mov     r10d, 1
        xor     r11d, r11d
.LBB0_2:
        add     r10, 1
        adc     r11, 0
        mov     rax, rcx
        mul     rbx
        imul    rcx, r14
        add     rcx, rdx
        imul    r9, rbx
        add     r9, rcx
        mov     qword ptr [rsp], rax
        mov     qword ptr [rsp + 8], r9
        mov     rcx, qword ptr [rsp]
        mov     r9, qword ptr [rsp + 8]
        cmp     r10, rdi
        mov     rax, r11
        sbb     rax, rsi
        mov     rbx, r10
        mov     r14, r11
        jb      .LBB0_2
.LBB0_3:
        add     rsp, 16
        pop     rbx
        pop     r14
        ret
 */
pub fn factorial_cpu(amount: u128) {
    let mut _result = 1_u128;
    for i in 1..amount {
        _result = std::hint::black_box(_result * i);
    }
}


pub fn fibonacci_cpu() {
    let mut a: u64 = 0;
    let mut b = 1;
    for _ in 0..10_000_000 {
        let c = std::hint::black_box(a + b);
        a = std::hint::black_box(b);
        b = std::hint::black_box(c);
    }
}


pub fn float_add() {
    let mut _x = 0.0;
    for _ in 0..10_000_000 {
        _x  = std::hint::black_box(_x + 0.139127123343);
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
    let mut _x = 0.0;
    for _ in 0..10_000_000 {
        _x  = std::hint::black_box(_x * 0.139127123343);
    }
}

pub fn float_division() {
    let mut _x = f64::MAX;
    for _ in 0..10_000_000 {
        _x  = std::hint::black_box(_x / 2.139127123343);
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
        Ok(
            OpenCLContext {
                platform,
                device,
                context,
                queue,
            }
        )
    }
}



pub struct OpenCLProgram {
    pub program: Program,
    pub kernel: Kernel,
    pub wg_size: Vec<usize>,
}


impl OpenCLProgram {
    pub fn new(context: &OpenCLContext, source: &str, kernel_name: &str, kernel_args: Vec<Vec<f32>>) -> Result<Self, String> {
        let program = Program::builder()
            .src(source)
            .devices(context.device)
            .build(&context.context)?;

        let wg_size = match context.device.info(DeviceInfo::MaxWorkItemSizes) {
            Ok(DeviceInfoResult::MaxWorkItemSizes(sizes)) => sizes,
            _ => return Err("Failed to get max work group size".to_string()),
        };


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
            self.kernel
                .cmd()
                .global_work_size((self.wg_size[0], self.wg_size[1], self.wg_size[2]))
                .enq()
        }
    }
}
