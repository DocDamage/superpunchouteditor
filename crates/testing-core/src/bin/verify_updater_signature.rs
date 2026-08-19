use base64::{engine::general_purpose::STANDARD, Engine as _};
use minisign_verify::{PublicKey, Signature};
use serde_json::Value;
use std::{
    env,
    error::Error,
    fs,
    io::{Error as IoError, ErrorKind},
    path::Path,
};

fn configured_public_key(config_path: &Path) -> Result<PublicKey, Box<dyn Error>> {
    let config: Value = serde_json::from_slice(&fs::read(config_path)?)?;
    let encoded = config
        .pointer("/plugins/updater/pubkey")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            IoError::new(
                ErrorKind::InvalidData,
                "tauri config is missing plugins.updater.pubkey",
            )
        })?;

    let decoded = STANDARD.decode(encoded)?;
    let minisign_text = String::from_utf8(decoded)?;
    Ok(PublicKey::decode(&minisign_text)?)
}

fn check_key(config_path: &Path) -> Result<(), Box<dyn Error>> {
    configured_public_key(config_path)?;
    println!(
        "Updater public key parsed successfully from {}",
        config_path.display()
    );
    Ok(())
}

fn verify(
    artifact_path: &Path,
    signature_path: &Path,
    config_path: &Path,
) -> Result<(), Box<dyn Error>> {
    let public_key = configured_public_key(config_path)?;
    let signature = Signature::from_file(signature_path)?;
    let artifact = fs::read(artifact_path)?;

    public_key.verify(&artifact, &signature, false)?;
    println!(
        "Updater signature verified for {} using the public key from {}",
        artifact_path.display(),
        config_path.display()
    );
    Ok(())
}

fn usage() -> &'static str {
    "usage:\n  verify-updater-signature check-key <tauri.conf.json>\n  verify-updater-signature verify <artifact> <artifact.sig> <tauri.conf.json>"
}

fn run() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    match args.as_slice() {
        [_, command, config] if command == "check-key" => check_key(Path::new(config)),
        [_, command, artifact, signature, config] if command == "verify" => {
            verify(Path::new(artifact), Path::new(signature), Path::new(config))
        }
        _ => Err(IoError::new(ErrorKind::InvalidInput, usage()).into()),
    }
}

fn main() {
    if let Err(error) = run() {
        eprintln!("Updater signature verification failed: {error}");
        std::process::exit(1);
    }
}
