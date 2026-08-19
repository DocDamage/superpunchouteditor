import { describe, expect, it } from "vitest";
import {
  buildTesterReport,
  createEmptyTesterReport,
  TEST_CHECKS,
} from "../components/TesterPanel";

describe("community tester report", () => {
  it("starts with every smoke-test item unchecked", () => {
    const state = createEmptyTesterReport();

    expect(TEST_CHECKS.every((check) => state.checks[check.id] === false)).toBe(true);
    expect(state.externalEmulator).toBe("not-tested");
  });

  it("generates a shareable privacy-safe Markdown summary", () => {
    const state = createEmptyTesterReport();
    state.testerName = "ExampleTester";
    state.appVersion = "2.0.0";
    state.windowsVersion = "Windows 11";
    state.installSource = "community tester kit";
    state.easeOfUse = "5";
    state.checks.installLaunch = true;
    state.checks.romLoad = true;
    state.notes = "Palette edit was easy to find.";

    const report = buildTesterReport(state);

    expect(report).toContain("# Super Punch-Out!! Editor Community Test Report");
    expect(report).toContain("Checklist: 2/9 completed");
    expect(report).toContain("Ease of use: 5/5");
    expect(report).toContain("- [x] Installer completed and the app launched normally");
    expect(report).toContain("Palette edit was easy to find.");
    expect(report).toContain("No ROM, SRAM/save-state, ROM path, or copyrighted game bytes");
  });
});
