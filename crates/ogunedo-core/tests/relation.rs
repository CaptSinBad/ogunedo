use ogunedo_core::{
    compute_target, negacyclic_mul, negacyclic_mul_naive, parameter_digest, parameters,
    relation_digest, statement_digest, verify_relation, PublicStatement, Witness,
    DEV_PARAMETERS_ID, DRAFT_PARAMETERS_ID, PROTOCOL_VERSION,
};

fn deterministic_witness(length: usize, bound: i32) -> Witness {
    let span = 2 * bound + 1;
    Witness {
        coeffs: (0..length)
            .map(|index| ((index as i32 * 7 + 3) % span) - bound)
            .collect(),
    }
}

fn hex(bytes: [u8; 32]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(64);
    for byte in bytes {
        out.push(DIGITS[(byte >> 4) as usize] as char);
        out.push(DIGITS[(byte & 0x0f) as usize] as char);
    }
    out
}

#[test]
fn ntt_matches_naive_for_all_registered_profiles() {
    for id in [DEV_PARAMETERS_ID, DRAFT_PARAMETERS_ID] {
        let p = parameters(id).unwrap();
        let a: Vec<u32> = (0..p.ring_degree)
            .map(|i| ((i * i + 17 * i + 3) as u32) % p.q)
            .collect();
        let b: Vec<u32> = (0..p.ring_degree)
            .map(|i| ((5 * i * i + 11) as u32) % p.q)
            .collect();
        assert_eq!(negacyclic_mul(&a, &b, p), negacyclic_mul_naive(&a, &b, p));
    }
}

#[test]
fn valid_relation_round_trip() {
    let p = parameters(DEV_PARAMETERS_ID).unwrap();
    let witness = deterministic_witness(p.columns * p.ring_degree, p.coefficient_bound);
    let seed = [0x42; 32];
    let target = compute_target(&seed, &witness, p).unwrap();
    let statement = PublicStatement {
        protocol_version: PROTOCOL_VERSION,
        parameter_id: p.id,
        matrix_seed: seed,
        target,
        context: [0x24; 32],
    };
    let receipt = verify_relation(&statement, &witness).unwrap();
    assert_eq!(
        receipt.public_values.statement_digest,
        statement_digest(&statement)
    );
    assert_eq!(receipt.public_values.relation_digest, relation_digest());
    assert_eq!(receipt.public_values.parameter_digest, parameter_digest(p));
}

#[test]
fn changed_target_is_rejected() {
    let p = parameters(DEV_PARAMETERS_ID).unwrap();
    let witness = deterministic_witness(p.columns * p.ring_degree, p.coefficient_bound);
    let seed = [0x11; 32];
    let mut target = compute_target(&seed, &witness, p).unwrap();
    target[0] = (target[0] + 1) % p.q;
    let statement = PublicStatement {
        protocol_version: PROTOCOL_VERSION,
        parameter_id: p.id,
        matrix_seed: seed,
        target,
        context: [0x22; 32],
    };
    assert!(verify_relation(&statement, &witness).is_err());
}

#[test]
fn changed_witness_is_rejected() {
    let p = parameters(DEV_PARAMETERS_ID).unwrap();
    let mut witness = deterministic_witness(p.columns * p.ring_degree, p.coefficient_bound);
    let seed = [0x55; 32];
    let target = compute_target(&seed, &witness, p).unwrap();
    let statement = PublicStatement {
        protocol_version: PROTOCOL_VERSION,
        parameter_id: p.id,
        matrix_seed: seed,
        target,
        context: [0x44; 32],
    };
    witness.coeffs[0] = if witness.coeffs[0] == p.coefficient_bound {
        witness.coeffs[0] - 1
    } else {
        witness.coeffs[0] + 1
    };
    assert!(verify_relation(&statement, &witness).is_err());
}

#[test]
fn coefficient_bound_is_enforced_before_equation_check() {
    let p = parameters(DEV_PARAMETERS_ID).unwrap();
    let mut witness = deterministic_witness(p.columns * p.ring_degree, p.coefficient_bound);
    witness.coeffs[0] = p.coefficient_bound + 1;
    assert!(compute_target(&[0u8; 32], &witness, p).is_err());
}

#[test]
fn digest_binds_context() {
    let p = parameters(DEV_PARAMETERS_ID).unwrap();
    let witness = deterministic_witness(p.columns * p.ring_degree, p.coefficient_bound);
    let target = compute_target(&[0x33; 32], &witness, p).unwrap();
    let mut left = PublicStatement {
        protocol_version: PROTOCOL_VERSION,
        parameter_id: p.id,
        matrix_seed: [0x33; 32],
        target,
        context: [0u8; 32],
    };
    let left_digest = statement_digest(&left);
    left.context[0] = 1;
    assert_ne!(left_digest, statement_digest(&left));
}

#[test]
fn registered_draft_profile_matches_v1_norm_bound() {
    let p = parameters(DRAFT_PARAMETERS_ID).unwrap();
    assert_eq!(p.q, 12_289);
    assert_eq!(p.ring_degree, 256);
    assert_eq!(p.rows, 1);
    assert_eq!(p.columns, 18);
    assert_eq!(p.coefficient_bound, 2);
    assert_eq!(p.l2_bound_squared, 10_240);
}

#[test]
fn canonical_digests_are_frozen_for_v1() {
    let dev = parameters(DEV_PARAMETERS_ID).unwrap();
    let draft = parameters(DRAFT_PARAMETERS_ID).unwrap();
    assert_eq!(
        hex(relation_digest()),
        "12ca32d006589f7fa7b3edaa4c1c19d3940661ae8528917738878dfe40d88feb"
    );
    assert_eq!(
        hex(parameter_digest(dev)),
        "bbf88444329630d91a6cade2f16681dcdd87eefc59e1b652ee799a1e77da97fe"
    );
    assert_eq!(
        hex(parameter_digest(draft)),
        "e8c8d261c2d5082a300d241434465245371db7c68b0d5df8e3a05f233af19611"
    );
}
