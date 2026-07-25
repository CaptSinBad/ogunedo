use sha2::{Digest, Sha256};

use crate::PublicStatement;

const STATEMENT_DOMAIN: &[u8] = b"OGUNEDO-STATEMENT-V1\0";
const RELATION_DOMAIN: &[u8] = b"OGUNEDO-KISIS-RELATION-V1\0";

pub fn relation_digest() -> [u8; 32] {
    let digest = Sha256::digest(RELATION_DOMAIN);
    digest.into()
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
