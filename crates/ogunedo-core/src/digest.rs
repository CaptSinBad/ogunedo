use sha2::{Digest, Sha256};

use crate::{ParameterStatus, Parameters, PublicStatement};

const STATEMENT_DOMAIN: &[u8] = b"OGUNEDO-STATEMENT-V1\0";
const RELATION_DOMAIN: &[u8] = b"OGUNEDO-KISIS-RELATION-V1\0";
const PARAMETER_DOMAIN: &[u8] = b"OGUNEDO-PARAMETERS-V1\0";

pub fn relation_digest() -> [u8; 32] {
    let digest = Sha256::digest(RELATION_DOMAIN);
    digest.into()
}

fn status_code(status: ParameterStatus) -> u32 {
    match status {
        ParameterStatus::DevelopmentOnly => 0,
        ParameterStatus::CryptanalysisRequired => 1,
        ParameterStatus::ProductionApproved => 2,
    }
}

pub fn parameter_digest(parameters: Parameters) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(PARAMETER_DOMAIN);
    hasher.update(parameters.id.to_le_bytes());
    hasher.update((parameters.name.len() as u64).to_le_bytes());
    hasher.update(parameters.name.as_bytes());
    hasher.update(parameters.q.to_le_bytes());
    hasher.update((parameters.ring_degree as u64).to_le_bytes());
    hasher.update((parameters.rows as u64).to_le_bytes());
    hasher.update((parameters.columns as u64).to_le_bytes());
    hasher.update(parameters.coefficient_bound.to_le_bytes());
    hasher.update(parameters.l2_bound_squared.to_le_bytes());
    hasher.update(parameters.primitive_root.to_le_bytes());
    hasher.update(status_code(parameters.status).to_le_bytes());
    hasher.finalize().into()
}

pub fn statement_digest(statement: &PublicStatement) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(STATEMENT_DOMAIN);
    hasher.update(statement.protocol_version.to_le_bytes());
    hasher.update(statement.parameter_id.to_le_bytes());
    hasher.update(statement.matrix_seed);
    hasher.update(statement.context);
    hasher.update((statement.target.len() as u64).to_le_bytes());
    for coefficient in &statement.target {
        hasher.update(coefficient.to_le_bytes());
    }
    hasher.finalize().into()
}
