#![forbid(unsafe_code)]

use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command as ProcessCommand,
    time::Duration,
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
use serde_json::json;
use sp1_sdk::{
    include_elf,
    network::{
        get_default_rpc_url_for_mode, get_explorer_url_for_mode,
        proto::GetProofRequestParamsResponse, signer::NetworkSigner, NetworkMode,
    },
    utils, Elf, HashableKey, ProveRequest, Prover, ProverClient, ProvingKey, SP1Proof,
    SP1ProofMode, SP1ProofWithPublicValues, SP1Stdin, SP1_CIRCUIT_VERSION,
};

const ELF: Elf = include_elf!("ogunedo-program");
const MAX_INSTANCE_BYTES: u64 = 64 * 1024 * 1024;
const MAX_PROOF_BYTES: u64 = 1024 * 1024 * 1024;
const PINNED_SP1_SDK_VERSION: &str = "6.2.2";
const PUBLIC_BENCHMARK_INSTANCE: &str = "fixtures/public-benchmark-instance.json";
const PROVE_ATOMIC_UNITS: u128 = 1_000_000_000_000_000_000;
const DEFAULT_NETWORK_TIMEOUT_SECS: u64 = 14_400;
const DEFAULT_AUCTION_TIMEOUT_SECS: u64 = 30;

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
        /// Mark the deterministic instance as intentionally public for remote proving.
        #[arg(long)]
        safe_for_remote_proving: bool,
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
    /// Build a sanitized Succinct Prover Network preflight report without submitting a request.
    NetworkEstimate {
        #[arg(long, default_value = PUBLIC_BENCHMARK_INSTANCE)]
        instance: PathBuf,
        #[arg(long, value_enum, default_value_t = ProofMode::Compressed)]
        mode: ProofMode,
        #[arg(long, default_value = "artifacts/network-preflight.json")]
        output: PathBuf,
        /// Maximum price per PGU in PROVE atomic units accepted for the request.
        #[arg(long)]
        max_price_per_pgu: u64,
        #[arg(long, default_value_t = DEFAULT_NETWORK_TIMEOUT_SECS)]
        timeout_secs: u64,
        #[arg(long, default_value_t = DEFAULT_AUCTION_TIMEOUT_SECS)]
        auction_timeout_secs: u64,
        #[arg(long, default_value_t = 1)]
        min_auction_period_secs: u64,
        #[arg(long)]
        allow_unreviewed_parameters: bool,
    },
    /// Submit a paid Succinct Prover Network request after exact approval.
    NetworkProve {
        #[arg(long, default_value = PUBLIC_BENCHMARK_INSTANCE)]
        instance: PathBuf,
        #[arg(long, value_enum)]
        mode: ProofMode,
        #[arg(long)]
        preflight: PathBuf,
        #[arg(long)]
        approval: String,
        #[arg(long)]
        output: PathBuf,
        #[arg(long)]
        manifest: PathBuf,
        #[arg(long)]
        receipt: PathBuf,
        #[arg(long, default_value = "artifacts/network-statement.json")]
        statement_output: PathBuf,
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

impl ProofMode {
    fn as_str(self) -> &'static str {
        match self {
            Self::Core => "core",
            Self::Compressed => "compressed",
            Self::Groth16 => "groth16",
        }
    }
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
    #[serde(default)]
    safe_for_remote_proving: bool,
    #[serde(default)]
    remote_proving_policy: Option<serde_json::Value>,
}

struct NetworkConfig {
    signer: NetworkSigner,
    requester_address: String,
    rpc_url: String,
    rpc_hostname: String,
    network_mode: NetworkMode,
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

fn write_json_atomic(path: &Path, value: &serde_json::Value) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, serde_json::to_vec_pretty(value)?)
        .with_context(|| format!("failed to write {}", tmp.display()))?;
    fs::rename(&tmp, path).with_context(|| {
        format!(
            "failed to atomically replace {} with {}",
            tmp.display(),
            path.display()
        )
    })?;
    Ok(())
}

fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    hex::encode(Sha256::digest(bytes))
}

fn file_sha256_hex(path: &Path) -> Result<String> {
    let bytes = fs::read(path).with_context(|| format!("failed to read {}", path.display()))?;
    Ok(sha256_hex(&bytes))
}

fn prove_amount_decimal(atomic_units: u128) -> String {
    let whole = atomic_units / PROVE_ATOMIC_UNITS;
    let frac = atomic_units % PROVE_ATOMIC_UNITS;
    format!("{whole}.{frac:018}")
}

fn rpc_hostname(rpc_url: &str) -> String {
    let without_scheme = rpc_url
        .split_once("://")
        .map(|(_, tail)| tail)
        .unwrap_or(rpc_url);
    without_scheme
        .split('/')
        .next()
        .unwrap_or(without_scheme)
        .to_string()
}

fn load_dotenv_for_network() -> Result<()> {
    let path = Path::new(".env");
    if !path.exists() {
        return Ok(());
    }

    let contents =
        fs::read_to_string(path).context("failed to read local .env for network process")?;
    for raw_line in contents.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        if !matches!(
            key,
            "SP1_PROVER" | "NETWORK_PRIVATE_KEY" | "NETWORK_RPC_URL"
        ) {
            continue;
        }
        if env::var_os(key).is_none() {
            let value = value
                .trim()
                .trim_matches('"')
                .trim_matches('\'')
                .to_string();
            env::set_var(key, value);
        }
    }
    Ok(())
}

fn network_config() -> Result<NetworkConfig> {
    load_dotenv_for_network()?;
    match env::var("SP1_PROVER") {
        Ok(value) if value == "network" => {}
        Ok(value) => {
            bail!("SP1_PROVER must be exactly 'network' for network commands, got '{value}'")
        }
        Err(_) => env::set_var("SP1_PROVER", "network"),
    }

    let private_key = env::var("NETWORK_PRIVATE_KEY")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .context("NETWORK_PRIVATE_KEY is not set for the local requester process")?;
    let signer = NetworkSigner::local(&private_key)
        .context("failed to parse NETWORK_PRIVATE_KEY for requester address derivation")?;
    let requester_address = format!("{:?}", signer.address());
    drop(private_key);

    let network_mode = NetworkMode::Mainnet;
    let rpc_url = env::var("NETWORK_RPC_URL")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| get_default_rpc_url_for_mode(network_mode));
    let rpc_hostname = rpc_hostname(&rpc_url);
    Ok(NetworkConfig {
        signer,
        requester_address,
        rpc_url,
        rpc_hostname,
        network_mode,
    })
}

fn sp1_mode(mode: ProofMode) -> Result<SP1ProofMode> {
    match mode {
        ProofMode::Core => bail!("network proving refuses core mode for the paid path"),
        ProofMode::Compressed => Ok(SP1ProofMode::Compressed),
        ProofMode::Groth16 => Ok(SP1ProofMode::Groth16),
    }
}

fn proof_has_mode(proof: &SP1ProofWithPublicValues, mode: ProofMode) -> bool {
    matches!(
        (&proof.proof, mode),
        (SP1Proof::Core(_), ProofMode::Core)
            | (SP1Proof::Compressed(_), ProofMode::Compressed)
            | (SP1Proof::Groth16(_), ProofMode::Groth16)
    )
}

fn public_benchmark_path_matches(path: &Path) -> Result<bool> {
    let expected = fs::canonicalize(PUBLIC_BENCHMARK_INSTANCE)
        .with_context(|| format!("{PUBLIC_BENCHMARK_INSTANCE} is missing"))?;
    let actual = fs::canonicalize(path)
        .with_context(|| format!("failed to canonicalize {}", path.display()))?;
    Ok(actual == expected)
}

fn enforce_remote_safe_instance(path: &Path, instance: &InstanceFile) -> Result<()> {
    if !public_benchmark_path_matches(path)? {
        bail!(
            "network proving is locked to {PUBLIC_BENCHMARK_INSTANCE}; refused {}",
            path.display()
        );
    }
    if !instance.safe_for_remote_proving {
        bail!("instance does not explicitly set safe_for_remote_proving=true");
    }
    let policy = instance
        .remote_proving_policy
        .as_ref()
        .context("remote-safe instance is missing remote_proving_policy")?;
    if policy
        .get("contains_trapdoor_secret")
        .and_then(serde_json::Value::as_bool)
        != Some(false)
        || policy
            .get("contains_signing_secret")
            .and_then(serde_json::Value::as_bool)
            != Some(false)
        || policy
            .get("contains_private_wallet_material")
            .and_then(serde_json::Value::as_bool)
            != Some(false)
    {
        bail!("remote proving policy does not explicitly classify secrets as absent");
    }
    Ok(())
}

fn json_string<'a>(value: &'a serde_json::Value, key: &str) -> Result<&'a str> {
    value
        .get(key)
        .and_then(serde_json::Value::as_str)
        .with_context(|| format!("preflight report missing string field '{key}'"))
}

fn save_proof_atomic(proof: &SP1ProofWithPublicValues, path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let tmp = path.with_extension("tmp");
    proof.save(&tmp)?;
    fs::rename(&tmp, path).with_context(|| {
        format!(
            "failed to atomically replace {} with {}",
            tmp.display(),
            path.display()
        )
    })?;
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
        safe_for_remote_proving: false,
        remote_proving_policy: None,
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
            safe_for_remote_proving,
        } => {
            let mut instance = deterministic_instance(parameters, &seed)?;
            instance.safe_for_remote_proving = safe_for_remote_proving;
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
            let client = ProverClient::builder().light().build().await;
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

            if env::var("SP1_PROVER").as_deref() == Ok("network") {
                bail!(
                    "local prove refuses SP1_PROVER=network; use `network-prove` for paid requests"
                );
            }
            let client = ProverClient::builder().cpu().build().await;
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
        Command::NetworkEstimate {
            instance,
            mode,
            output,
            max_price_per_pgu,
            timeout_secs,
            auction_timeout_secs,
            min_auction_period_secs,
            allow_unreviewed_parameters,
        } => {
            let instance_file = read_instance(&instance)?;
            enforce_remote_safe_instance(&instance, &instance_file)?;
            let receipt =
                verify_relation(&instance_file.statement, require_witness(&instance_file)?)?;
            if !is_production_approved(receipt.parameters) && !allow_unreviewed_parameters {
                bail!(
                    "parameter set '{}' is {:?}; network proving requires --allow-unreviewed-parameters until cryptanalysis is complete",
                    receipt.parameters.name,
                    receipt.parameters.status
                );
            }
            let selected_sp1_mode = sp1_mode(mode)?;
            let config = network_config()?;
            let client = ProverClient::builder()
                .network_for(config.network_mode)
                .signer(config.signer.clone())
                .rpc_url(&config.rpc_url)
                .build()
                .await;
            let proving_key = client.setup(ELF).await?;
            let vkey = format!("{:?}", proving_key.verifying_key().bytes32());
            let elf_sha256 = sha256_hex(&ELF);

            let light = ProverClient::builder().light().build().await;
            let (execution_output, execution_report) =
                light.execute(ELF, stdin_for(&instance_file)?).await?;
            let execution_values: PublicValues = bincode::deserialize(execution_output.as_slice())
                .context("invalid execution public values during network preflight")?;
            check_public_values(&execution_values, &instance_file.statement)?;
            let cycle_limit = execution_report.total_instruction_count();
            let gas_limit = execution_report.gas().unwrap_or(cycle_limit);

            let params = client.get_proof_request_params(selected_sp1_mode).await?;
            let (network_base_fee, default_max_price_per_pgu, domain_len) = match params {
                GetProofRequestParamsResponse::Auction(auction) => (
                    auction
                        .base_fee
                        .parse::<u64>()
                        .context("network returned invalid base_fee")?,
                    auction
                        .max_price_per_pgu
                        .parse::<u64>()
                        .context("network returned invalid max_price_per_pgu")?,
                    auction.domain.len(),
                ),
                GetProofRequestParamsResponse::Unsupported => {
                    bail!("network proof parameter query is unsupported for the selected network")
                }
            };
            let balance = client.get_balance().await?;
            let max_spend_atomic = network_base_fee as u128
                + (gas_limit as u128).saturating_mul(max_price_per_pgu as u128);
            let max_spend = prove_amount_decimal(max_spend_atomic);
            let statement_digest_hex = hex::encode(statement_digest(&instance_file.statement));
            let approval_phrase = format!("APPROVE OGUNEDO NETWORK PROOF UP TO {max_spend} PROVE");
            let explorer_url = get_explorer_url_for_mode(config.network_mode);
            let report = json!({
                "schema": "ogunedo-network-preflight-v1",
                "submitted": false,
                "requires_owner_approval": true,
                "approval_phrase": approval_phrase,
                "requester_public_address": config.requester_address,
                "network": "mainnet",
                "rpc_endpoint_hostname": config.rpc_hostname,
                "explorer_base_url": explorer_url,
                "sp1_sdk_version": PINNED_SP1_SDK_VERSION,
                "sp1_circuit_version": SP1_CIRCUIT_VERSION,
                "proof_mode": mode.as_str(),
                "guest_elf_sha256": elf_sha256,
                "vkey": vkey,
                "statement_digest": format!("0x{statement_digest_hex}"),
                "relation_digest": format!("0x{}", hex::encode(relation_digest())),
                "parameter_digest": format!("0x{}", hex::encode(parameter_digest(receipt.parameters))),
                "fixture": PUBLIC_BENCHMARK_INSTANCE,
                "fixture_safe_for_remote_proving": true,
                "public_benchmark_witness_visibility": "witness may be visible to ordinary network provers",
                "cycle_limit": cycle_limit,
                "gas_limit": gas_limit,
                "network_base_fee_atomic_prove": network_base_fee.to_string(),
                "network_default_max_price_per_pgu_atomic_prove": default_max_price_per_pgu.to_string(),
                "requested_max_price_per_pgu_atomic_prove": max_price_per_pgu.to_string(),
                "max_possible_prove_spend_atomic": max_spend_atomic.to_string(),
                "max_possible_prove_spend": max_spend,
                "request_timeout_secs": timeout_secs,
                "auction_timeout_secs": auction_timeout_secs,
                "min_auction_period_secs": min_auction_period_secs,
                "auction_domain_length": domain_len,
                "requester_balance_atomic_prove": balance.to_string(),
                "source_commit": option_env!("GIT_COMMIT").unwrap_or("not-embedded"),
                "private_key_recorded": false
            });
            write_json_atomic(&output, &report)?;
            println!("wrote sanitized network preflight to {}", output.display());
            println!("requester public address: {}", config.requester_address);
            println!("network: mainnet");
            println!("proof mode: {}", mode.as_str());
            println!(
                "guest ELF SHA-256: {}",
                json_string(&report, "guest_elf_sha256")?
            );
            println!("vkey: {}", json_string(&report, "vkey")?);
            println!(
                "statement digest: {}",
                json_string(&report, "statement_digest")?
            );
            println!(
                "maximum possible spend: {} PROVE",
                json_string(&report, "max_possible_prove_spend")?
            );
            println!("exact approval phrase required before submission:");
            println!("{}", json_string(&report, "approval_phrase")?);
        }
        Command::NetworkProve {
            instance,
            mode,
            preflight,
            approval,
            output,
            manifest,
            receipt,
            statement_output,
            allow_unreviewed_parameters,
        } => {
            let preflight_json: serde_json::Value = serde_json::from_slice(
                &fs::read(&preflight)
                    .with_context(|| format!("failed to read {}", preflight.display()))?,
            )
            .with_context(|| format!("invalid JSON in {}", preflight.display()))?;
            let required_approval = json_string(&preflight_json, "approval_phrase")?;
            if approval != required_approval {
                bail!("exact approval phrase mismatch; paid request not submitted");
            }
            if json_string(&preflight_json, "proof_mode")? != mode.as_str() {
                bail!("requested proof mode does not match preflight report");
            }

            let instance_file = read_instance(&instance)?;
            enforce_remote_safe_instance(&instance, &instance_file)?;
            let relation_receipt =
                verify_relation(&instance_file.statement, require_witness(&instance_file)?)?;
            if !is_production_approved(relation_receipt.parameters) && !allow_unreviewed_parameters
            {
                bail!(
                    "parameter set '{}' is {:?}; network proving requires --allow-unreviewed-parameters until cryptanalysis is complete",
                    relation_receipt.parameters.name,
                    relation_receipt.parameters.status
                );
            }
            let selected_sp1_mode = sp1_mode(mode)?;
            let config = network_config()?;
            let client = ProverClient::builder()
                .network_for(config.network_mode)
                .signer(config.signer.clone())
                .rpc_url(&config.rpc_url)
                .build()
                .await;
            let proving_key = client.setup(ELF).await?;
            let vkey = format!("{:?}", proving_key.verifying_key().bytes32());
            let elf_sha256 = sha256_hex(&ELF);
            if json_string(&preflight_json, "guest_elf_sha256")? != elf_sha256 {
                bail!("current guest ELF hash differs from preflight");
            }
            if json_string(&preflight_json, "vkey")? != vkey {
                bail!("current vkey differs from preflight");
            }
            if json_string(&preflight_json, "statement_digest")?
                != format!(
                    "0x{}",
                    hex::encode(statement_digest(&instance_file.statement))
                )
            {
                bail!("current statement digest differs from preflight");
            }
            let max_price_per_pgu =
                json_string(&preflight_json, "requested_max_price_per_pgu_atomic_prove")?
                    .parse::<u64>()
                    .context("invalid max_price_per_pgu in preflight")?;
            let requester_balance = json_string(&preflight_json, "requester_balance_atomic_prove")?
                .parse::<u128>()
                .context("invalid requester_balance_atomic_prove in preflight")?;
            let max_possible_spend =
                json_string(&preflight_json, "max_possible_prove_spend_atomic")?
                    .parse::<u128>()
                    .context("invalid max_possible_prove_spend_atomic in preflight")?;
            if requester_balance < max_possible_spend {
                bail!(
                    "requester network balance ({requester_balance} atomic PROVE) is below maximum possible spend ({max_possible_spend} atomic PROVE); paid request not submitted"
                );
            }
            let timeout_secs = preflight_json
                .get("request_timeout_secs")
                .and_then(serde_json::Value::as_u64)
                .context("preflight missing request_timeout_secs")?;
            let auction_timeout_secs = preflight_json
                .get("auction_timeout_secs")
                .and_then(serde_json::Value::as_u64)
                .context("preflight missing auction_timeout_secs")?;
            let min_auction_period_secs = preflight_json
                .get("min_auction_period_secs")
                .and_then(serde_json::Value::as_u64)
                .context("preflight missing min_auction_period_secs")?;
            let cycle_limit = preflight_json
                .get("cycle_limit")
                .and_then(serde_json::Value::as_u64)
                .context("preflight missing cycle_limit")?;
            let gas_limit = preflight_json
                .get("gas_limit")
                .and_then(serde_json::Value::as_u64)
                .context("preflight missing gas_limit")?;

            let mut public_statement = instance_file.clone();
            public_statement.witness = None;
            write_instance(&statement_output, &public_statement)?;

            let stdin = stdin_for(&instance_file)?;
            let start_time = std::time::SystemTime::now();
            let request_id = client
                .prove(&proving_key, stdin.clone())
                .mode(selected_sp1_mode)
                .timeout(Duration::from_secs(timeout_secs))
                .cycle_limit(cycle_limit)
                .gas_limit(gas_limit)
                .min_auction_period(min_auction_period_secs)
                .auction_timeout(Duration::from_secs(auction_timeout_secs))
                .max_price_per_pgu(max_price_per_pgu)
                .request()
                .await
                .context("network proof request failed")?;
            let request_id_hex = format!("{:?}", request_id);
            let explorer = format!(
                "{}/request/{}",
                get_explorer_url_for_mode(config.network_mode),
                request_id_hex
            );
            let submitted_at_unix = start_time
                .duration_since(std::time::UNIX_EPOCH)
                .ok()
                .map(|d| d.as_secs());
            let pending_receipt_json = json!({
                "schema": "ogunedo-network-receipt-v1",
                "status": "submitted",
                "request_id": request_id_hex,
                "explorer_url": explorer,
                "requester_public_address": config.requester_address,
                "network": "mainnet",
                "rpc_endpoint_hostname": config.rpc_hostname,
                "proof_mode": mode.as_str(),
                "submitted_at_unix": submitted_at_unix,
                "completed_at_unix": null,
                "requested_max_price_per_pgu_atomic_prove": max_price_per_pgu.to_string(),
                "cycle_limit": cycle_limit,
                "gas_limit": gas_limit,
                "proof_sha256": null,
                "proof_size_bytes": null,
                "immediate_verification_result": false,
                "fresh_process_verification_result": false,
                "private_key_recorded": false
            });
            write_json_atomic(&receipt, &pending_receipt_json)?;
            println!("submitted network proof request: {}", request_id_hex);
            println!("explorer: {}", explorer);

            let proof = client
                .wait_proof(
                    request_id,
                    Some(Duration::from_secs(timeout_secs)),
                    Some(Duration::from_secs(auction_timeout_secs)),
                )
                .await
                .context("network proof did not complete")?;
            if !proof_has_mode(&proof, mode) {
                bail!("downloaded proof mode does not match requested mode");
            }
            client.verify(&proof, proving_key.verifying_key(), None)?;
            let values = decode_public_values(&proof)?;
            check_public_values(&values, &instance_file.statement)?;
            save_proof_atomic(&proof, &output)?;
            let proof_sha256 = file_sha256_hex(&output)?;
            let proof_size = fs::metadata(&output)?.len();

            let fresh_status = ProcessCommand::new(env::current_exe()?)
                .arg("verify")
                .arg("--proof")
                .arg(&output)
                .arg("--statement")
                .arg(&statement_output)
                .env_remove("NETWORK_PRIVATE_KEY")
                .env_remove("NETWORK_RPC_URL")
                .env_remove("SP1_PROVER")
                .status()
                .context("failed to launch credential-free verifier process")?;
            if !fresh_status.success() {
                bail!("fresh credential-free verification process failed");
            }

            let completion_time = std::time::SystemTime::now();
            let manifest_json = json!({
                "schema": "ogunedo-network-proof-manifest-v1",
                "proof_path": output.display().to_string(),
                "proof_mode": mode.as_str(),
                "proof_sha256": proof_sha256,
                "proof_size_bytes": proof_size,
                "statement_path": statement_output.display().to_string(),
                "statement_digest": format!("0x{}", hex::encode(statement_digest(&instance_file.statement))),
                "guest_elf_sha256": elf_sha256,
                "vkey": vkey,
                "sp1_sdk_version": PINNED_SP1_SDK_VERSION,
                "sp1_circuit_version": SP1_CIRCUIT_VERSION,
                "request_id": request_id_hex,
                "explorer_url": explorer,
                "immediate_verification_result": true,
                "fresh_process_verification_result": true,
                "private_key_recorded": false
            });
            write_json_atomic(&manifest, &manifest_json)?;
            let receipt_json = json!({
                "schema": "ogunedo-network-receipt-v1",
                "status": "completed",
                "request_id": json_string(&manifest_json, "request_id")?,
                "explorer_url": json_string(&manifest_json, "explorer_url")?,
                "requester_public_address": config.requester_address,
                "network": "mainnet",
                "rpc_endpoint_hostname": config.rpc_hostname,
                "proof_mode": mode.as_str(),
                "submitted_at_unix": start_time.duration_since(std::time::UNIX_EPOCH).ok().map(|d| d.as_secs()),
                "completed_at_unix": completion_time.duration_since(std::time::UNIX_EPOCH).ok().map(|d| d.as_secs()),
                "requested_max_price_per_pgu_atomic_prove": max_price_per_pgu.to_string(),
                "cycle_limit": cycle_limit,
                "gas_limit": gas_limit,
                "proof_sha256": json_string(&manifest_json, "proof_sha256")?,
                "proof_size_bytes": proof_size,
                "immediate_verification_result": true,
                "fresh_process_verification_result": true,
                "private_key_recorded": false
            });
            write_json_atomic(&receipt, &receipt_json)?;
            println!("network proof downloaded and verified");
            println!("request id: {}", json_string(&manifest_json, "request_id")?);
            println!("explorer: {}", json_string(&manifest_json, "explorer_url")?);
            println!("proof: {}", output.display());
            println!("manifest: {}", manifest.display());
            println!("receipt: {}", receipt.display());
        }
        Command::Verify { proof, statement } => {
            let instance = read_instance(&statement)?;
            let client = ProverClient::builder().cpu().build().await;
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
            let client = ProverClient::builder().cpu().build().await;
            let proving_key = client.setup(ELF).await?;
            println!("{:?}", proving_key.verifying_key().bytes32());
        }
    }

    Ok(())
}
