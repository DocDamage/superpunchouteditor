# Project Format v2

Project format v2 is the canonical persistent editing-session format for the 2.0 application line.

## Storage

A project is a directory. The canonical manifest is `project-v2.json`; the previous valid manifest may be retained as `project-v2.recovery.json`. Project-owned imported assets may be stored below `assets/`, and generated patches may be stored below `patches/`.

## Required manifest state

The v2 document records:

- schema version, independently versioned from the application;
- application version that wrote the file;
- project metadata;
- base-ROM SHA-1, byte size, detected region and optional display filename;
- the complete serialized canonical `EditJournal`;
- project settings that affect materialization;
- duplicated/relocated-bank metadata;
- imported-asset references, hashes and optional embedded copies;
- optional project thumbnail;
- saved/current revision information;
- expected current materialized SHA-1;
- write timestamp;
- an integrity SHA-1 over the serialized document.

The base ROM bytes are never embedded.

## Save transaction

1. Snapshot one journal revision.
2. Validate imported-asset paths and hashes.
3. Serialize an integrity-protected envelope to a temporary file in the project directory.
4. Re-read and verify the temporary manifest.
5. Preserve the previous valid manifest as the recovery file.
6. Atomically persist the verified manifest where the platform allows.

A project save is not allowed to manufacture a clean ROM-export state; project-save state and ROM-export state are distinct concepts.

## Load transaction

1. Parse the envelope without mutating application state.
2. Verify document integrity.
3. Require/select a base ROM outside the project file.
4. Verify base hash and size.
5. Validate every journal operation against the immutable base and preceding operations.
6. Materialize in scratch memory.
7. Verify the expected current SHA-1.
8. Replace the active journal/session only after every check succeeds.

The existing session remains intact on load failure.

## v1 migration

The legacy v1 format stored edit metadata but generally did not store replacement payload bytes. A v1 project containing edit records therefore cannot safely reconstruct those edits. The application must report this explicitly and may offer metadata-only import; it must not claim that the editing session was restored.

A v1 project with no edit records may migrate its metadata into an empty v2 journal.

## Security constraints

- no absolute or parent-traversal asset paths;
- bounded asset/file sizes before decoding;
- embedded asset hashes verified before use;
- base ROM supplied by the user and never copied into the project bundle automatically;
- malformed or incompatible documents fail before state replacement.
