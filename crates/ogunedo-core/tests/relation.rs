use ogunedo_core::{
    compute_target, negacyclic_mul, negacyclic_mul_naive, parameters, statement_digest,
    verify_relation, PublicStatement, Witness, DEV_PARAMETERS_ID, DRAFT_PARAMETERS_ID,
    PROTOCOL_VERSION,
};

fn deterministic_witness(length: usize, bound: i32) -> Witness {
    let span = 2 * bound + 1;
    Witness {
        coeffs: (0..length)
            .map(|index| ((index as i32 * 7 + 3) % span) - bound)
            .collect(),
    }
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
    assert_eq!(receipt.public_values.statement_digest, statement_digest(&statement));
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
