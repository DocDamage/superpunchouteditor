use patch_core::{apply_bps, apply_ips, generate_bps, generate_ips_bytes, BpsMetadata};
use project_core::{load_project_v2, save_project_v2, ProjectDocumentV2, ProjectMetadata};
use rom_core::{EditRequest, Rom, RomSession};

#[test]
fn journal_save_patch_and_project_restore_same_bytes() {
    let base_rom = Rom::new((0u8..=127).collect());
    let mut session = RomSession::from_rom(&base_rom, Some("synthetic.sfc".to_string()));
    session
        .commit(
            "synthetic mixed edit",
            vec![
                EditRequest::WriteBytes {
                    offset: 4,
                    after: vec![0xaa, 0xbb, 0xcc],
                    asset_id: Some("palette/test".to_string()),
                    description: Some("palette bytes".to_string()),
                },
                EditRequest::WriteBytes {
                    offset: 40,
                    after: vec![9, 8, 7, 6, 5],
                    asset_id: Some("sprite/test".to_string()),
                    description: Some("sprite bytes".to_string()),
                },
            ],
        )
        .unwrap();

    let materialized = session.materialize().unwrap();
    assert_ne!(materialized.base_sha1, materialized.current_sha1);

    let ips = generate_ips_bytes(session.base().bytes(), &materialized.bytes).unwrap();
    let ips_result = apply_ips(session.base().bytes(), &ips).unwrap();
    assert_eq!(ips_result, materialized.bytes);

    let bps = generate_bps(
        session.base().bytes(),
        &materialized.bytes,
        &BpsMetadata::default(),
    )
    .unwrap();
    let bps_result = apply_bps(session.base().bytes(), &bps).unwrap();
    assert_eq!(bps_result, materialized.bytes);

    let project_dir = tempfile::tempdir().unwrap();
    let document = ProjectDocumentV2::from_session(
        "2.0.0",
        ProjectMetadata::default(),
        &session,
        Some("synthetic.sfc".to_string()),
    )
    .unwrap();
    save_project_v2(project_dir.path(), &document).unwrap();
    let restored_document = load_project_v2(project_dir.path()).unwrap();
    restored_document
        .validate_against_base(session.base())
        .unwrap();

    let mut restored = RomSession::from_rom(&base_rom, Some("synthetic.sfc".to_string()));
    restored.replace_journal(restored_document.journal).unwrap();
    let restored_materialized = restored.materialize().unwrap();

    assert_eq!(restored_materialized.bytes, materialized.bytes);
    assert_eq!(
        restored_materialized.current_sha1,
        materialized.current_sha1
    );
    assert_eq!(ips_result, bps_result);
    assert_eq!(bps_result, restored_materialized.bytes);
}
