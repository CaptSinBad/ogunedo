use alloc::vec::Vec;
use sha2::{Digest, Sha256};

use crate::Parameters;

const MATRIX_DOMAIN: &[u8] = b"OGUNEDO-MATRIX-EXPAND-V1\0";

pub fn derive_matrix_polynomial(
    seed: &[u8; 32],
    row: usize,
    column: usize,
    parameters: Parameters,
) -> Vec<u32> {
    assert!(parameters.q <= u16::MAX as u32, "parameter modulus exceeds expander width");

    let q = parameters.q;
    let rejection_limit = (u16::MAX as u32 + 1) - ((u16::MAX as u32 + 1) % q);
    let mut output = Vec::with_capacity(parameters.ring_degree);
    let mut block_counter = 0u64;

    while output.len() < parameters.ring_degree {
        let mut hasher = Sha256::new();
        hasher.update(MATRIX_DOMAIN);
        hasher.update(parameters.id.to_le_bytes());
        hasher.update(seed);
        hasher.update((row as u32).to_le_bytes());
        hasher.update((column as u32).to_le_bytes());
        hasher.update(block_counter.to_le_bytes());
        let block = hasher.finalize();

        for pair in block.chunks_exact(2) {
            let candidate = u16::from_le_bytes([pair[0], pair[1]]) as u32;
            if candidate < rejection_limit {
                output.push(candidate % q);
                if output.len() == parameters.ring_degree {
                    break;
                }
            }
        }

        block_counter = block_counter.checked_add(1).expect("matrix expander counter overflow");
    }

    output
}

pub fn derive_matrix(seed: &[u8; 32], parameters: Parameters) -> Vec<Vec<u32>> {
    let mut matrix = Vec::with_capacity(parameters.rows * parameters.columns);
    for row in 0..parameters.rows {
        for column in 0..parameters.columns {
            matrix.push(derive_matrix_polynomial(seed, row, column, parameters));
        }
    }
    matrix
}
