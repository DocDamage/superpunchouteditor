# Plugin Security and Trust Model

## Stable 2.0 decision

Plugins are **research-blocked** and excluded from the stable product surface. Stable IPC commands for plugin discovery, load/unload, enable/disable, command execution, script execution, batch execution, reload and plugin-directory access return an explicit disabled error.

The previous design is not considered a security sandbox because unrestricted scripting facilities and discovery-time execution can cross the editor's trust boundary. Hiding the UI alone is not sufficient; stable backend execution is disabled as well.

## Requirements before re-enabling plugins

A future experimental plugin system must satisfy all of the following before it can be considered for stable release:

1. **Metadata without execution** — plugin ID/name/version/permissions/hash must be parsed without running top-level plugin code.
2. **Unique identity** — validated plugin IDs are unique; placeholder IDs such as `loading` are invalid.
3. **Disabled by default** — discovery never executes code and a newly discovered plugin remains disabled until explicit user action.
4. **Trust prompt** — first enablement shows source path, plugin ID, cryptographic file hash and requested capabilities. A file-hash change invalidates prior trust.
5. **Least privilege** — preferred implementation is a separate constrained process with a narrow message API. If in-process Lua is retained, OS, I/O, package loading, debug and other unnecessary libraries are excluded.
6. **Resource bounds** — execution time/instruction count, memory, output size and IPC payload size are bounded.
7. **Filesystem/network isolation** — no filesystem, process, shell, network or URL access without a specific user-granted capability and a backend-enforced scope.
8. **No shell interpolation** — external tools receive executable paths and argument arrays directly; user/plugin strings are never concatenated into a shell command.
9. **Failure containment** — timeout, crash and malformed response cannot corrupt the active `RomSession`; plugin-proposed edits are validated and committed through the canonical journal transaction API.
10. **Tests** — startup, discovery, duplicate ID, enable/disable, changed hash, denied capability, timeout, memory/output bounds and crash recovery tests must pass.

## Native plugins

Native or otherwise unrestricted plugins are arbitrary-code execution. If such a mechanism is ever offered, the UI and documentation must say so explicitly and must not describe it as sandboxed.

## Current dependency position

The unused legacy Wasmtime plugin runtime is removed from the stable dependency graph rather than carrying known-vulnerable code for a hypothetical future feature. Any future runtime choice requires a fresh security review and dependency audit.
