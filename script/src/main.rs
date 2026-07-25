#![forbid(unsafe_code)]

use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use ogunedo_core::{
    compute_target, is_production_approved, parameter_digest, parameters, relation_digest,
    statement_digest, verify_relation, PublicStatement, PublicValues, Witness, DEV_PARAMETERS_ID,
    DRAFT_PARAMETERS_ID, PROTOCOL_VERSION,
};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;
use serde::{Deserialize, Serialize};
use sp1_sdk::{
    include_elf, utils, Elf, HashableKey, ProveRequest, Prover, ProverClient, ProvingKey,
    SP1ProofWithPublicValues, SP1Stdin,
};

const ELF: Elf = include_elf!("ogunedo-program");
const MAX_INSTANCE_BYTES: u64 = 64 * 1024 * 1024;
const MAX_PROOF_BYTES: u64 = 1024 * 1024 * 1024;

#[derive(Debug, Parser)]
#[command(
    name = "ogunedo",
    version,
    about = "Ogunedo K-ISIS SP1 proof-of-knowledge CLI"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Generate a deterministic valid instance and private witness.
    Generate {
        #[arg(long, value_enum, default_value_t = ParameterChoice::Dev)]
        parameters: ParameterChoice,
        #[arg(long, default_value = "ogunedo-fixture")]
        seed: String,
        #[arg(long)]
        output: PathBuf,
    },
    /// Remove the private witness and write a verifier-safe public statement file.
    Redact {
        #[arg(long)]
        instance: PathBuf,
        #[arg(long)]
        output: PathBuf,
    },
    /// Verify the relation natively without generating a proof.
    Check {
        #[arg(long)]
        instance: PathBuf,
    },
    /// Execute the SP1 program and report cycle count.
    Execute {
        #[arg(long)]
        instance: PathBuf,
    },
    /// Generate and locally verify an SP1 proof.
    Prove {
        #[arg(long)]
        instance: PathBuf,
        #[arg(long, value_enum, default_value_t = ProofMode::Compressed)]
        mode: ProofMode,
        #[arg(long)]
        output: PathBuf,
        /// Required for parameter sets that have not completed independent cryptanalysis.
        #[arg(long)]
        allow_unreviewed_parameters: bool,
    },
    /// Verify a saved proof and bind it to the supplied public statement.
    Verify {
        #[arg(long)]
        proof: PathBuf,
        #[arg(long)]
        statement: PathBuf,
    },
    /// Print the SP1 verification-key commitment for this program.
    Vkey,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum ProofMode {
    Core,
    Compressed,
    Groth16,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum ParameterChoice {
    Dev,
    Draft,
}

impl ParameterChoice {
    fn id(self) -> u32 {
        match self {
            Self::Dev => DEV_PARAMETERS_ID,
            Self::Draft => DRAFT_PARAMETERS_ID,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct InstanceFile {
    statement: PublicStatement,
    witness: Option<Witness>,
}

fn enforce_file_limit(path: &Path, maximum: u64, kind: &str) -> Result<()> {
    let size = fs::metadata(path)
        .with_context(|| format!("failed to stat {}", path.display()))?
        .len();
    if size > maximum {
        bail!("{kind} file is {size} bytes, exceeding the {maximum}-byte limit");
    }
    Ok(())
}

fn read_instance(path: &Path) -> Result<InstanceFile> {
    enforce_file_limit(path, MAX_INSTANCE_BYTES, "instance")?;
    let bytes = fs::read(path).with_context(|| format!("failed to read {}", path.display()))?;
    serde_json::from_slice(&bytes).with_context(|| format!("invalid JSON in {}", path.display()))
}

fn write_instance(path: &Path, instance: &InstanceFile) -> Result<()> {
    use std::io::Write;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let bytes = serde_json::to_vec_pretty(instance)?;
    let mut options = fs::OpenOptions::new();
    options.create(true).truncate(true).write(true);
    #[cfg(unix)]
    let mode = if instance.witness.is_some() {
        0o600
    } else {
        0o644
    };
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(mode);
    }
    let mut file = options
        .open(path)
        .with_context(|| format!("failed to open {}", path.display()))?;
    file.write_all(&bytes)
        .with_context(|| format!("failed to write {}", path.display()))?;
    file.sync_all()
        .with_context(|| format!("failed to sync {}", path.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(mode))
            .with_context(|| format!("failed to set permissions on {}", path.display()))?;
    }
    Ok(())
}

fn require_witness(instance: &InstanceFile) -> Result<&Witness> {
    instance
        .witness
        .as_ref()
        .context("instance does not contain the private witness")
}

fn stdin_for(instance: &InstanceFile) -> Result<SP1Stdin> {
    let witness = require_witness(instance)?;
    let mut stdin = SP1Stdin::new();
    stdin.write(&instance.statement);
    stdin.write(witness);
    Ok(stdin)
}

fn check_public_values(values: &PublicValues, statement: &PublicStatement) -> Result<()> {
    if values.protocol_version != PROTOCOL_VERSION {
        bail!(
            "proof committed unsupported protocol version {}",
            values.protocol_version
        );
    }
    if values.parameter_id != statement.parameter_id {
        bail!("proof parameter id does not match statement");
    }
    let selected =
        parameters(statement.parameter_id).context("proof committed unknown parameter set")?;
    if values.statement_digest != statement_digest(statement) {
        bail!("proof is valid for a different public statement");
    }
    if values.relation_digest != relation_digest() {
        bail!("proof relation-domain digest does not match Ogunedo K-ISIS v1");
    }
    if values.parameter_digest != parameter_digest(selected) {
        bail!("proof parameter digest does not match the registered parameter set");
    }
    Ok(())
}

fn decode_public_values(proof: &SP1ProofWithPublicValues) -> Result<PublicValues> {
    let bytes = proof.public_values.as_slice();
    let values: PublicValues =
        bincode::deserialize(bytes).context("invalid Ogunedo public-value encoding")?;
    let encoded_size = bincode::serialized_size(&values)? as usize;
    if encoded_size != bytes.len() {
        bail!("Ogunedo public values contain trailing bytes");
    }
    Ok(values)
}

fn deterministic_instance(choice: ParameterChoice, seed_text: &str) -> Result<InstanceFile> {
    use sha2::{Digest, Sha256};

    let selected = parameters(choice.id()).context("parameter set missing")?;
    let root_seed: [u8; 32] = Sha256::digest(seed_text.as_bytes()).into();
    let matrix_seed: [u8; 32] = Sha256::digest([b"matrix".as_slice(), &root_seed].concat()).into();
    let context: [u8; 32] = Sha256::digest([b"context".as_slice(), &root_seed].concat()).into();
    let mut rng = ChaCha20Rng::from_seed(root_seed);
    let witness = Witness {
        coeffs: (0..selected.columns * selected.ring_degree)
            .map(|_| rng.gen_range(-selected.coefficient_bound..=selected.coefficient_bound))
            .collect(),
    };
    let target = compute_target(&matrix_seed, &witness, selected)?;
    Ok(InstanceFile {
        statement: PublicStatement {
            protocol_version: PROTOCOL_VERSION,
            parameter_id: selected.id,
            matrix_seed,
            target,
            context,
        },
        witness: Some(witness),
    })
}

#[tokio::main]
async fn main() -> Result<()> {
    utils::setup_logger();
    let cli = Cli::parse();

    match cli.command {
        Command::Generate {
            parameters,
            seed,
            output,
        } => {
            let instance = deterministic_instance(parameters, &seed)?;
            verify_relation(&instance.statement, require_witness(&instance)?)?;
            write_instance(&output, &instance)?;
            println!("wrote private witness instance to {}", output.display());
            println!("protect this file; use `ogunedo redact` before sharing it");
            println!(
                "statement digest: 0x{}",
                hex::encode(statement_digest(&instance.statement))
            );
        }
        Command::Redact { instance, output } => {
            let mut instance = read_instance(&instance)?;
            instance.witness = None;
            write_instance(&output, &instance)?;
            println!("wrote public statement to {}", output.display());
            println!(
                "statement digest: 0x{}",
                hex::encode(statement_digest(&instance.statement))
            );
        }
        Command::Check { instance } => {
            let instance = read_instance(&instance)?;
            let receipt = verify_relation(&instance.statement, require_witness(&instance)?)?;
            println!("relation accepted");
            println!("parameter set: {}", receipt.parameters.name);
            println!(
                "statement digest: 0x{}",
                hex::encode(receipt.public_values.statement_digest)
            );
        }
        Command::Execute { instance } => {
            let instance = read_instance(&instance)?;
            verify_relation(&instance.statement, require_witness(&instance)?)?;
            let client = ProverClient::from_env().await;
            let (output, report) = client.execute(ELF, stdin_for(&instance)?).await?;
            let bytes = output.as_slice();
            let values: PublicValues =
                bincode::deserialize(bytes).context("invalid execution public values")?;
            if bincode::serialized_size(&values)? as usize != bytes.len() {
                bail!("execution public values contain trailing bytes");
            }
            check_public_values(&values, &instance.statement)?;
            println!(
                "execution accepted in {} cycles",
                report.total_instruction_count()
            );
            println!(
                "statement digest: 0x{}",
                hex::encode(values.statement_digest)
            );
        }
        Command::Prove {
            instance,
            mode,
            output,
            allow_unreviewed_parameters,
        } => {
            let instance = read_instance(&instance)?;
            let receipt = verify_relation(&instance.statement, require_witness(&instance)?)?;
            if !is_production_approved(receipt.parameters) && !allow_unreviewed_parameters {
                bail!(
                    "parameter set '{}' is {:?}; repeat with --allow-unreviewed-parameters only for research",
                    receipt.parameters.name,
                    receipt.parameters.status
                );
            }

            let client = ProverClient::from_env().await;
            let proving_key = client.setup(ELF).await?;
            let stdin = stdin_for(&instance)?;
            let proof = match mode {
                ProofMode::Core => client.prove(&proving_key, stdin).core().await?,
                ProofMode::Compressed => client.prove(&proving_key, stdin).compressed().await?,
                ProofMode::Groth16 => client.prove(&proving_key, stdin).groth16().await?,
            };
            client.verify(&proof, proving_key.verifying_key(), None)?;
            let values = decode_public_values(&proof)?;
            check_public_values(&values, &instance.statement)?;
            if let Some(parent) = output.parent() {
                fs::create_dir_all(parent)?;
            }
            proof.save(&output)?;
            println!("proof generated and verified: {}", output.display());
            println!("mode: {:?}", mode);
            println!("vkey: {:?}", proving_key.verifying_key().bytes32());
            println!(
                "statement digest: 0x{}",
                hex::encode(values.statement_digest)
            );
        }
        Command::Verify { proof, statement } => {
            let instance = read_instance(&statement)?;
            let client = ProverClient::from_env().await;
            let proving_key = client.setup(ELF).await?;
            enforce_file_limit(&proof, MAX_PROOF_BYTES, "proof")?;
            let proof = SP1ProofWithPublicValues::load(&proof)?;
            client.verify(&proof, proving_key.verifying_key(), None)?;
            let values = decode_public_values(&proof)?;
            check_public_values(&values, &instance.statement)?;
            println!("proof verified and bound to statement");
            println!(
                "statement digest: 0x{}",
                hex::encode(values.statement_digest)
            );
        }
        Command::Vkey => {
            let client = ProverClient::from_env().await;
            let proving_key = client.setup(ELF).await?;
            println!("{:?}", proving_key.verifying_key().bytes32());
        }
    }

    Ok(())
}
