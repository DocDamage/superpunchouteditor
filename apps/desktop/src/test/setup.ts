import '@testing-library/jest-dom';

// Mock Tauri APIs that are unavailable in jsdom.
// Each mock is intentionally minimal — tests override behaviour per-case via vi.mocked().
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
  isTauri: vi.fn(() => false),
}));

vi.mock('@tauri-apps/plugin-dialog', () => ({
  open: vi.fn(),
}));
