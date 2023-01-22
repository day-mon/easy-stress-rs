pub fn sqrt_cpu() {
    let _ = (952.0_f32).sqrt();
}

pub fn factorial() {
    let mut _factorial = 1;
    for i in 1..=100 {
        _factorial *= i;
    }
}


pub fn fibonacci() {
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


