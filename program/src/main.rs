#![no_main]
sp1_zkvm::entrypoint!(main);

use ogunedo_core::{verify_relation, PublicStatement, Witness};

pub fn main() {
    let statement = sp1_zkvm::io::read::<PublicStatement>();
    let witness = sp1_zkvm::io::read::<Witness>();

    let receipt = verify_relation(&statement, &witness)
        .unwrap_or_else(|error| panic!("Ogunedo K-ISIS relation rejected: {error}"));

    sp1_zkvm::io::commit(&receipt.public_values);
}
