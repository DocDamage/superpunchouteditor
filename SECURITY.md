# Security Policy

## Supported versions

Security fixes are applied to the current supported 2.0 development/release line and any stable release explicitly named in release notes. Experimental and research-blocked features are not security-supported APIs and may be removed without compatibility guarantees.

## Reporting a vulnerability

Do not open a public issue containing exploit details, private ROM data, signing material, local filesystem paths, or other sensitive evidence. Use GitHub's private vulnerability reporting/security-advisory flow for this repository when available. Include the affected version/commit, reproduction steps, expected/actual behavior, and the minimum proof needed to demonstrate impact.

## Security boundaries

### ROM and project data

User-supplied ROMs are local data. Public CI and repository fixtures must use synthetic data; copyrighted ROM contents must not be committed or uploaded by automated verification. Project format v2 stores ROM identity and the edit journal, never the base ROM bytes.

### Filesystem

Automatic writes are restricted to application-owned locations or explicit user-selected destinations. Logical filenames from projects/layout packs must reject absolute paths and parent traversal. File sizes and image dimensions are bounded before decoding.

### WebView / IPC

Production builds use a restrictive Content Security Policy and a least-privilege main-window capability. Custom Tauri commands validate inputs at the command/domain boundary. Stable UI commands must be registered mechanically; gated experimental calls are recorded explicitly in the command-contract manifest.

### Plugins

Plugins are disabled in stable builds. The previous Lua/WASM design did not provide a sufficient security sandbox. Stable IPC does not load, execute, reload, batch-run, or browse plugin code. Re-enabling plugins requires a reviewed trust/capability design with explicit opt-in, real plugin identity/hash verification, no discovery-time execution, restricted standard libraries, bounded execution/memory, and permission prompts for sensitive capabilities.

### Updates and release signing

Stable releases require signed installers/updater artifacts, validated updater metadata/signatures, published checksums, and an SBOM. Private signing keys must never be committed to the repository and should be stored offline or in appropriately protected release secrets.

## Dependency response

RustSec and npm high-severity audits are merge/release gates. Unused vulnerable dependencies are removed rather than retained for hypothetical future features. Exceptions require a documented impact analysis and target removal/update date.
