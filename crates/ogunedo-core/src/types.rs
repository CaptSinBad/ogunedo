use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

pub const PROTOCOL_VERSION: u32 = 1;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicStatement {
    pub protocol_version: u32,
    pub parameter_id: u32,
    pub matrix_seed: [u8; 32],
    pub target: Vec<u32>,
    pub context: [u8; 32],
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Witness {
    /// Column-major signed coefficients. Polynomial j occupies
    /// `coeffs[j * ring_degree .. (j + 1) * ring_degree]`.
    pub coeffs: Vec<i32>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicValues {
    pub protocol_version: u32,
    pub parameter_id: u32,
    pub statement_digest: [u8; 32],
    pub relation_digest: [u8; 32],
}
