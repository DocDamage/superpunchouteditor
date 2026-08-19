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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::{
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
        time::{SystemTime, UNIX_EPOCH},
    };

    // Public, prehashed Minisign example vector from minisign-verify's MIT-licensed
    // documentation. The production verifier still reads the application's own key.
    const PUBLIC_KEY_BASE64: &str =
        "RWQf6LRCGA9i53mlYecO4IzT51TGPpvWucNSCh1CBM0QTaLn73Y7GFO3";
    const SIGNATURE: &str = concat!(
        "untrusted comment: signature from minisign secret key\n",
        "RUQf6LRCGA9i559r3g7V1qNyJDApGip8MfqcadIgT9CuhV3EMhHoN1mGTkUidF/zSrlQgXdy8ofjb7bNJJylDOocrCo8KLzZwo=\n",
        "trusted comment: timestamp:1633700835\tfile:test\tprehashed\n",
        "wLMDjy9FLAuxZ3q4NlEvkgtyhrr0gtTu6KC4KBJdITbbOeAi1zBIYo0v4iTgt8jJpIidRJnp94ABQkJAgAooBQ==\n",
    );

    static FIXTURE_COUNTER: AtomicU64 = AtomicU64::new(0);

    struct Fixture {
        dir: PathBuf,
        artifact: PathBuf,
        signature: PathBuf,
        config: PathBuf,
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.dir);
        }
    }

    fn fixture(contents: &[u8]) -> Fixture {
        let unique = format!(
            "spo-updater-signature-{}-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time must be after Unix epoch")
                .as_nanos(),
            FIXTURE_COUNTER.fetch_add(1, Ordering::Relaxed)
        );
        let dir = env::temp_dir().join(unique);
        fs::create_dir_all(&dir).expect("fixture directory must be created");

        let artifact = dir.join("artifact.bin");
        let signature = dir.join("artifact.bin.sig");
        let config = dir.join("tauri.conf.json");

        fs::write(&artifact, contents).expect("artifact fixture must be written");
        fs::write(&signature, SIGNATURE).expect("signature fixture must be written");

        let minisign_key = format!(
            "untrusted comment: minisign public key\n{PUBLIC_KEY_BASE64}\n"
        );
        let config_json = json!({
            "plugins": {
                "updater": {
                    "pubkey": STANDARD.encode(minisign_key.as_bytes())
                }
            }
        });
        fs::write(
            &config,
            serde_json::to_vec(&config_json).expect("config fixture must serialize"),
        )
        .expect("config fixture must be written");

        Fixture {
            dir,
            artifact,
            signature,
            config,
        }
    }

    #[test]
    fn valid_prehashed_signature_is_accepted() {
        let fixture = fixture(b"test");
        verify(&fixture.artifact, &fixture.signature, &fixture.config)
            .expect("known-good prehashed signature must verify");
    }

    #[test]
    fn tampered_artifact_is_rejected() {
        let fixture = fixture(b"tost");
        assert!(
            verify(&fixture.artifact, &fixture.signature, &fixture.config).is_err(),
            "tampering must invalidate the signature"
        );
    }
}
