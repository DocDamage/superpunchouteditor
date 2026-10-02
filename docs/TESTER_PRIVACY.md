# Tester Privacy and Copyright Boundary

Community testing should collect only the information needed to reproduce editor behavior.

Safe to share:

- app version and source commit;
- Windows version;
- installer/output SHA-256 hashes;
- UI screenshots that do not expose sensitive local paths;
- editor error messages and non-copyrighted logs;
- steps to reproduce;
- the in-app Tester Checklist Markdown report.

Do not share:

- ROM files or ROM bytes;
- SRAM or emulator save states;
- extracted copyrighted game assets/data;
- emulator binaries unless redistribution rights are independently established;
- signing keys, certificates, tokens, or passwords.

The community tester kit and its report workflow are designed around these boundaries.

## What the editor keeps on your computer

- The path of the last ROM you opened, so it can be reopened at startup.
- An automatic copy of your edit journal per ROM, in the app's own data folder. It contains your changes, not the ROM.
- Your emulator choice. To find an emulator the editor reads file names in the usual download, desktop, documents and program folders; it does not open or send those files anywhere.

None of this leaves the computer.
