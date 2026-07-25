use alloc::vec;
use alloc::vec::Vec;
use core::fmt;

use crate::{
    derive_matrix_polynomial, negacyclic_mul, parameter_digest, parameters, relation_digest,
    statement_digest, ParameterStatus, Parameters, PublicStatement, PublicValues, Witness,
    PROTOCOL_VERSION,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RelationError {
    UnsupportedProtocolVersion(u32),
    UnknownParameterSet(u32),
    ParameterArithmeticInvalid,
    TargetLength {
        expected: usize,
        actual: usize,
    },
    NonCanonicalTarget {
        index: usize,
        value: u32,
    },
    WitnessLength {
        expected: usize,
        actual: usize,
    },
    CoefficientOutOfRange {
        index: usize,
        value: i32,
        bound: i32,
    },
    L2BoundExceeded {
        actual: u64,
        maximum: u64,
    },
    EquationMismatch {
        index: usize,
        expected: u32,
        actual: u32,
    },
}

impl fmt::Display for RelationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedProtocolVersion(version) => {
                write!(formatter, "unsupported protocol version {version}")
            }
            Self::UnknownParameterSet(id) => write!(formatter, "unknown parameter set 0x{id:08x}"),
            Self::ParameterArithmeticInvalid => {
                write!(formatter, "parameter arithmetic is invalid")
            }
            Self::TargetLength { expected, actual } => {
                write!(formatter, "target length {actual}, expected {expected}")
            }
            Self::NonCanonicalTarget { index, value } => {
                write!(
                    formatter,
                    "target coefficient {index} is non-canonical: {value}"
                )
            }
            Self::WitnessLength { expected, actual } => {
                write!(formatter, "witness length {actual}, expected {expected}")
            }
            Self::CoefficientOutOfRange {
                index,
                value,
                bound,
            } => write!(
                formatter,
                "witness coefficient {index}={value} exceeds absolute bound {bound}"
            ),
            Self::L2BoundExceeded { actual, maximum } => {
                write!(formatter, "witness squared norm {actual} exceeds {maximum}")
            }
            Self::EquationMismatch {
                index,
                expected,
                actual,
            } => write!(
                formatter,
                "module equation mismatch at coefficient {index}: expected {expected}, got {actual}"
            ),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for RelationError {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerificationReceipt {
    pub parameters: Parameters,
    pub public_values: PublicValues,
}

fn mul_mod(left: u32, right: u32, q: u32) -> u32 {
    ((left as u64 * right as u64) % q as u64) as u32
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

fn is_prime(value: u32) -> bool {
    if value < 2 {
        return false;
    }
    if value.is_multiple_of(2) {
        return value == 2;
    }
    let mut divisor = 3u32;
    while divisor as u64 * divisor as u64 <= value as u64 {
        if value.is_multiple_of(divisor) {
            return false;
        }
        divisor += 2;
    }
    true
}

fn validate_parameters(parameters: Parameters) -> Result<(), RelationError> {
    if !parameters.ring_degree.is_power_of_two()
        || parameters.rows == 0
        || parameters.columns == 0
        || parameters.q < 3
        || parameters.q > u16::MAX as u32
        || !is_prime(parameters.q)
        || !((parameters.q - 1) as usize).is_multiple_of(2 * parameters.ring_degree)
        || parameters.coefficient_bound < 0
        || parameters.l2_bound_squared == 0
        || parameters.primitive_root == 0
        || parameters.primitive_root >= parameters.q
    {
        return Err(RelationError::ParameterArithmeticInvalid);
    }

    let order = 2 * parameters.ring_degree;
    let psi = mod_pow(
        parameters.primitive_root,
        ((parameters.q - 1) as usize / order) as u64,
        parameters.q,
    );
    if mod_pow(psi, parameters.ring_degree as u64, parameters.q) != parameters.q - 1
        || mod_pow(psi, order as u64, parameters.q) != 1
    {
        return Err(RelationError::ParameterArithmeticInvalid);
    }
    Ok(())
}

fn signed_to_mod_q(value: i32, q: u32) -> u32 {
    (value as i64).rem_euclid(q as i64) as u32
}

pub fn compute_target(
    matrix_seed: &[u8; 32],
    witness: &Witness,
    parameters: Parameters,
) -> Result<Vec<u32>, RelationError> {
    validate_parameters(parameters)?;
    let expected_witness_length = parameters.columns * parameters.ring_degree;
    if witness.coeffs.len() != expected_witness_length {
        return Err(RelationError::WitnessLength {
            expected: expected_witness_length,
            actual: witness.coeffs.len(),
        });
    }

    let mut squared_norm = 0u64;
    for (index, &coefficient) in witness.coeffs.iter().enumerate() {
        let magnitude = coefficient.unsigned_abs();
        if magnitude > parameters.coefficient_bound as u32 {
            return Err(RelationError::CoefficientOutOfRange {
                index,
                value: coefficient,
                bound: parameters.coefficient_bound,
            });
        }
        squared_norm = squared_norm
            .checked_add(magnitude as u64 * magnitude as u64)
            .ok_or(RelationError::L2BoundExceeded {
                actual: u64::MAX,
                maximum: parameters.l2_bound_squared,
            })?;
    }
    if squared_norm > parameters.l2_bound_squared {
        return Err(RelationError::L2BoundExceeded {
            actual: squared_norm,
            maximum: parameters.l2_bound_squared,
        });
    }

    let n = parameters.ring_degree;
    let q = parameters.q;
    let mut output = vec![0u32; parameters.rows * n];

    for row in 0..parameters.rows {
        let row_output = &mut output[row * n..(row + 1) * n];
        for column in 0..parameters.columns {
            let matrix_polynomial = derive_matrix_polynomial(matrix_seed, row, column, parameters);
            let witness_polynomial: Vec<u32> = witness.coeffs[column * n..(column + 1) * n]
                .iter()
                .map(|&value| signed_to_mod_q(value, q))
                .collect();
            let product = negacyclic_mul(&matrix_polynomial, &witness_polynomial, parameters);
            for (accumulator, product_coefficient) in row_output.iter_mut().zip(product) {
                let sum = *accumulator + product_coefficient;
                *accumulator = if sum >= q { sum - q } else { sum };
            }
        }
    }

    Ok(output)
}

pub fn verify_relation(
    statement: &PublicStatement,
    witness: &Witness,
) -> Result<VerificationReceipt, RelationError> {
    if statement.protocol_version != PROTOCOL_VERSION {
        return Err(RelationError::UnsupportedProtocolVersion(
            statement.protocol_version,
        ));
    }
    let selected = parameters(statement.parameter_id)
        .ok_or(RelationError::UnknownParameterSet(statement.parameter_id))?;
    validate_parameters(selected)?;

    let expected_target_length = selected.rows * selected.ring_degree;
    if statement.target.len() != expected_target_length {
        return Err(RelationError::TargetLength {
            expected: expected_target_length,
            actual: statement.target.len(),
        });
    }
    for (index, &coefficient) in statement.target.iter().enumerate() {
        if coefficient >= selected.q {
            return Err(RelationError::NonCanonicalTarget {
                index,
                value: coefficient,
            });
        }
    }

    let computed = compute_target(&statement.matrix_seed, witness, selected)?;
    for (index, (&expected, &actual)) in statement.target.iter().zip(&computed).enumerate() {
        if expected != actual {
            return Err(RelationError::EquationMismatch {
                index,
                expected,
                actual,
            });
        }
    }

    let public_values = PublicValues {
        protocol_version: PROTOCOL_VERSION,
        parameter_id: selected.id,
        statement_digest: statement_digest(statement),
        relation_digest: relation_digest(),
        parameter_digest: parameter_digest(selected),
    };

    Ok(VerificationReceipt {
        parameters: selected,
        public_values,
    })
}

pub fn is_production_approved(parameters: Parameters) -> bool {
    matches!(parameters.status, ParameterStatus::ProductionApproved)
}
