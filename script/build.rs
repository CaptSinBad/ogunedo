use std::path::PathBuf;

fn main() {
    let mut args = sp1_build::BuildArgs::default();
    if let Some(cargo_home) = cargo_home() {
        args.rustflags.push(format!(
            "--remap-path-prefix={}=/root/.cargo",
            cargo_home.display()
        ));
    }
    sp1_build::build_program_with_args("../program", args);
}

fn cargo_home() -> Option<PathBuf> {
    std::env::var_os("CARGO_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".cargo")))
}
