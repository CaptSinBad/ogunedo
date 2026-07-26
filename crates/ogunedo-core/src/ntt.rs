use alloc::vec;
use alloc::vec::Vec;

use crate::Parameters;

fn add_mod(a: u32, b: u32, q: u32) -> u32 {
    let sum = a + b;
    if sum >= q {
        sum - q
    } else {
        sum
    }
}

fn sub_mod(a: u32, b: u32, q: u32) -> u32 {
    if a >= b {
        a - b
    } else {
        a + q - b
    }
}

fn mul_mod(a: u32, b: u32, q: u32) -> u32 {
    ((a as u64 * b as u64) % q as u64) as u32
}

fn mod_pow(mut base: u32, mut exponent: u64, q: u32) -> u32 {
    let mut result = 1u32;
    while exponent != 0 {
        if exponent & 1 == 1 {
            result = mul_mod(result, base, q);
        }
        base = mul_mod(base, base, q);
        exponent >>= 1;
    }
    result
}

fn mod_inverse(value: u32, q: u32) -> u32 {
    mod_pow(value, (q - 2) as u64, q)
}

fn cyclic_ntt(values: &mut [u32], root: u32, invert: bool, q: u32) {
    let n = values.len();
    debug_assert!(n.is_power_of_two());

    let mut j = 0usize;
    for i in 1..n {
        let mut bit = n >> 1;
        while j & bit != 0 {
            j ^= bit;
            bit >>= 1;
        }
        j ^= bit;
        if i < j {
            values.swap(i, j);
        }
    }

    let selected_root = if invert { mod_inverse(root, q) } else { root };
    let mut length = 2usize;
    while length <= n {
        let step_root = mod_pow(selected_root, (n / length) as u64, q);
        for start in (0..n).step_by(length) {
            let mut twiddle = 1u32;
            for offset in 0..length / 2 {
                let even = values[start + offset];
                let odd = mul_mod(values[start + offset + length / 2], twiddle, q);
                values[start + offset] = add_mod(even, odd, q);
                values[start + offset + length / 2] = sub_mod(even, odd, q);
                twiddle = mul_mod(twiddle, step_root, q);
            }
        }
        length <<= 1;
    }

    if invert {
        let n_inverse = mod_inverse(n as u32, q);
        for value in values {
            *value = mul_mod(*value, n_inverse, q);
        }
    }
}

pub fn negacyclic_mul(a: &[u32], b: &[u32], parameters: Parameters) -> Vec<u32> {
    let n = parameters.ring_degree;
    assert_eq!(a.len(), n, "left polynomial length mismatch");
    assert_eq!(b.len(), n, "right polynomial length mismatch");
    assert_eq!(
        (parameters.q - 1) as usize % (2 * n),
        0,
        "2N must divide q-1"
    );

    let q = parameters.q;
    let psi = mod_pow(
        parameters.primitive_root,
        ((q - 1) as usize / (2 * n)) as u64,
        q,
    );
    let psi_inverse = mod_inverse(psi, q);
    let omega = mul_mod(psi, psi, q);

    let mut left = vec![0u32; n];
    let mut right = vec![0u32; n];
    let mut twist = 1u32;
    for i in 0..n {
        left[i] = mul_mod(a[i] % q, twist, q);
        right[i] = mul_mod(b[i] % q, twist, q);
        twist = mul_mod(twist, psi, q);
    }

    cyclic_ntt(&mut left, omega, false, q);
    cyclic_ntt(&mut right, omega, false, q);
    for i in 0..n {
        left[i] = mul_mod(left[i], right[i], q);
    }
    cyclic_ntt(&mut left, omega, true, q);

    let mut inverse_twist = 1u32;
    for value in &mut left {
        *value = mul_mod(*value, inverse_twist, q);
        inverse_twist = mul_mod(inverse_twist, psi_inverse, q);
    }
    left
}

pub fn negacyclic_mul_naive(a: &[u32], b: &[u32], parameters: Parameters) -> Vec<u32> {
    let n = parameters.ring_degree;
    assert_eq!(a.len(), n);
    assert_eq!(b.len(), n);
    let q = parameters.q as i64;
    let mut accumulator = vec![0i64; n];

    for (i, &left) in a.iter().enumerate() {
        for (j, &right) in b.iter().enumerate() {
            let product = (left as i64 * right as i64) % q;
            let degree = i + j;
            if degree < n {
                accumulator[degree] += product;
            } else {
                accumulator[degree - n] -= product;
            }
        }
    }

    accumulator
        .into_iter()
        .map(|value| value.rem_euclid(q) as u32)
        .collect()
}
