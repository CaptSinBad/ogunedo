#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]

extern crate alloc;

mod digest;
mod matrix;
mod ntt;
mod params;
mod relation;
mod types;

pub use digest::{parameter_digest, relation_digest, statement_digest};
pub use matrix::{derive_matrix, derive_matrix_polynomial};
pub use ntt::{negacyclic_mul, negacyclic_mul_naive};
pub use params::{parameters, ParameterStatus, Parameters, DEV_PARAMETERS_ID, DRAFT_PARAMETERS_ID};
pub use relation::{
    compute_target, is_production_approved, verify_relation, RelationError, VerificationReceipt,
};
pub use types::{PublicStatement, PublicValues, Witness, PROTOCOL_VERSION};
