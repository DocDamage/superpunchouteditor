# Community Windows Testing Guide

This guide is for people testing an unsigned Windows build of **Super Punch-Out!! Editor** before a production release.

## What you need

- Windows 11 x64.
- The tester-kit ZIP supplied by the project owner.
- Your **own legally obtained** Super Punch-Out!! ROM (`.sfc` or `.smc`).
- About 10–20 minutes for the basic smoke test.
- Optional: a locally installed SNES emulator for the external-emulator test.

The tester kit does **not** include a ROM, SRAM/save state, emulator binary, certificate/private key, or signing secret.

## Before you run it

1. Extract the tester-kit ZIP to a normal writable folder.
2. Read `CHECKSUMS.txt` and verify the installer SHA-256 if the file is present.
3. Keep a backup of your original ROM. The test procedure always saves edited output to a **new file**.
4. The community tester installer may be unsigned. Windows can therefore identify the publisher as unknown. Only test a build you received from the project owner and whose SHA-256 matches the value shipped in the kit. If you are not comfortable running an unsigned build, do not continue.

PowerShell can calculate a file hash without modifying the file:

```powershell
Get-FileHash -Algorithm SHA256 '.\Super Punch-Out Editor Setup.exe'
```

## The 10-minute smoke test

### 1. Install and launch

Install the app normally and launch it.

**Pass:** the app opens without immediately crashing or showing a blank window.

### 2. Follow the first-run screen

The first screen should make the next action obvious. Choose **Open My ROM** or **Open ROM**.

**Pass:** you can tell what to do without reading developer documentation.

### 3. Open your ROM

Choose your local `.sfc` or `.smc` file and confirm the detected region.

**Pass:** the editor recognizes the supported ROM and lands on a stable editing screen. A stable tester build must not unexpectedly drop you into a hidden experimental tool.

### 4. Make one obvious reversible edit

Pick a boxer and make a small visual change, preferably a palette/color edit.

- Note the original value.
- Make the change.
- Click **Undo**.
- Click **Redo**.

**Pass:** Undo restores the original value and Redo restores the edit.

### 5. Test the current revision

Choose **Test Game** from the main workflow.

**Pass:** the test path uses the current edited revision rather than the untouched base ROM or an older saved copy.

If embedded emulation cannot run because a compatible local core is not configured, record that clearly. Do not claim it passed.

### 6. Save without touching the original ROM

Save/export the edited ROM to a **new filename**. Never choose your original source ROM as the output for this test.

**Pass:** the new output contains the edit and the original file remains unchanged.

### 7. Save and reopen a project

Use **Projects** to save the editing session. Close the editor, reopen the project, and confirm the same logical edit is present.

**Pass:** the project restores the edit journal and produces the same edited revision.

### 8. Try one safe-cancel path

Start a save/export operation and cancel it once, or exercise overwrite protection without approving the overwrite.

**Pass:** cancellation is non-destructive and the UI makes it clear that no file was changed.

### 9. Fill out the Tester Checklist

Open **Tester Checklist** from the bottom of the left sidebar.

- Check only items you actually tested.
- Rate how easy the editor was to understand.
- Write down anything confusing, even if it eventually worked.
- Use **Copy Report** or **Download .md**.

The checklist is stored locally on that PC for convenience.

## What feedback is most useful

Report:

- the exact app version or tester-kit source;
- your Windows version;
- what you clicked immediately before the problem;
- what you expected;
- what actually happened;
- whether it happens every time;
- screenshots of the editor UI when useful;
- non-copyrighted logs or error text;
- hashes of test artifacts when requested.

For usability feedback, specifically call out:

- any point where you did not know what to click next;
- labels that did not mean what you expected;
- buttons you could not find;
- warnings that were unclear;
- screens that felt overloaded;
- actions that looked dangerous even when they were safe;
- anything that required instructions when it should have been self-explanatory.

## Never send these in a bug report

Do **not** upload or attach:

- `.sfc` or `.smc` ROM files;
- SRAM files;
- emulator save states;
- screenshots containing a local ROM path if you consider that path private;
- commercial game data extracted from the ROM;
- private signing keys, certificates, or passwords.

Use observations, screenshots of the editor, hashes, filenames, and non-copyrighted logs instead.

## Optional advanced canonical-output verification

The tester kit may include the PowerShell helpers from `scripts/windows/acceptance-*.ps1`. These are for testers who want to verify saved-ROM/BPS/IPS/project-restored byte equivalence without sharing ROM data.

Start with `WINDOWS_ACCEPTANCE.md` for that deeper process. The scripts record metadata and hashes; they do not need to upload ROM bytes.

## External emulator test

This test is optional for community testers.

If you already have a local emulator:

1. Open the editor's emulator/external-tool settings.
2. Select the emulator executable already installed on your PC.
3. Launch it through the editor after making an edit.
4. Make a second reversible edit and launch again.

**Pass:** each launch receives the current edited revision and the editor does not copy the emulator binary into its installation or project.

## Uninstall test

If you are willing to test uninstall behavior:

1. Save a project and preferences.
2. Uninstall the application normally.
3. Reinstall the same tester build.
4. Confirm project/preferences data was not unexpectedly destroyed by the default uninstall path.

## Release status of a community tester build

A community tester build is **not a production stable release**. In particular, an unsigned smoke/test installer is not evidence that production Authenticode or Tauri updater signing is configured.

Production release approval remains separate and requires signed artifacts, updater verification, checksums, SBOM output, and the full Windows release gates.
