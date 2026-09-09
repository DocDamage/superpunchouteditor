# Publishing a Community Tester Build

Use this short owner checklist when sharing a Windows tester build.

- Confirm the source revision is on a branch/commit you intend people to test.
- Confirm frontend tests/typecheck/build are green.
- Confirm Windows Package Smoke is green for the same source revision when applicable.
- Confirm Community Windows Tester Kit is green.
- Download `super-punch-out-editor-community-tester-kit` from that successful workflow run.
- Verify the ZIP contains `README_FIRST.txt`, `START_HERE.md`, `CHECKSUMS.txt`, `BUILD_INFO.json`, the NSIS installer, and `advanced-evidence/`.
- Verify no ROM, SRAM/save-state, or emulator binary is present.
- Share the tester-kit ZIP as-is rather than sending a loose installer with no provenance/checksum.
- Tell testers to begin with `START_HERE.md` and use the in-app **Tester Checklist**.
- Ask testers to return the generated Markdown report or open a **Community test report** GitHub issue.
- Do not call the tester build a stable release or a signed production build.
- Do not mark real-ROM acceptance PASS until evidence from actual Windows real-ROM testing exists.
