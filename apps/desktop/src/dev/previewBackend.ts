/**
 * Development-only UI preview backend.
 *
 * `npm run dev` serves the interface in a normal browser, where the Rust
 * backend does not exist. Opening `http://localhost:1420/?preview` installs
 * this stand-in so screens can be designed and checked with made-up sample
 * data. It is loaded only in development builds (see main.tsx) and is never
 * part of a packaged app.
 *
 * Everything here is synthetic: invented boxers, generated colours and
 * simple shapes drawn in code. It contains no game data.
 */

import { mockIPC, mockWindows } from "@tauri-apps/api/mocks";

interface Rgb {
  r: number;
  g: number;
  b: number;
}

interface Edit {
  label: string;
  before: Rgb[];
  after: Rgb[];
  boxer: string;
}

const SAMPLE_SHA1 = "0000000000000000000000000000000000000000";
const SAMPLE_PATH = "C:\\Preview\\Sample Game (Preview).sfc";

const BOXERS = ["Sample Boxer A", "Sample Boxer B", "Sample Boxer C", "Sample Boxer D", "Sample Boxer E"];

const keyOf = (name: string): string => name.toLowerCase().replace(/[^a-z0-9]+/g, "_");

const snap = (value: number): number => {
  const five = Math.round((value / 255) * 31);
  return (five << 3) | (five >> 2);
};

const rgb = (r: number, g: number, b: number): Rgb => ({ r: snap(r), g: snap(g), b: snap(b) });

function samplePalette(seed: number): Rgb[] {
  const outfit: Rgb[] = [
    [rgb(200, 30, 20), rgb(140, 20, 10)],
    [rgb(30, 90, 210), rgb(20, 50, 140)],
    [rgb(30, 160, 70), rgb(20, 100, 40)],
    [rgb(230, 180, 20), rgb(160, 110, 10)],
    [rgb(130, 60, 200), rgb(80, 30, 130)],
  ][seed % 5];
  const row: Rgb[] = [
    rgb(0, 0, 132), // see-through
    rgb(40, 20, 10),
    rgb(132, 82, 41),
    rgb(190, 130, 90),
    rgb(247, 198, 156),
    outfit[0],
    outfit[1],
    rgb(255, 255, 255),
    rgb(132, 132, 132),
    rgb(60, 60, 60),
    rgb(240, 240, 100),
    rgb(100, 180, 240),
    rgb(20, 20, 20),
    rgb(180, 180, 180),
    rgb(90, 50, 30),
    rgb(250, 120, 60),
  ];
  return [...row, ...row.map((color) => ({ ...color }))];
}

const palettes = new Map<string, Rgb[]>();
const originals = new Map<string, Rgb[]>();
BOXERS.forEach((name, index) => {
  palettes.set(keyOf(name), samplePalette(index));
  originals.set(keyOf(name), samplePalette(index));
});

const undoStack: Edit[] = [];
const redoStack: Edit[] = [];
let romLoaded = false;
let emulatorPath = "";

const css = ({ r, g, b }: Rgb): string => `rgb(${r}, ${g}, ${b})`;

function pngBytes(size: number, draw: (ctx: CanvasRenderingContext2D) => void): number[] {
  const canvas = document.createElement("canvas");
  canvas.width = size;
  canvas.height = size;
  const ctx = canvas.getContext("2d");
  if (!ctx) return [];
  ctx.imageSmoothingEnabled = false;
  draw(ctx);
  const binary = atob(canvas.toDataURL("image/png").split(",")[1]);
  return Array.from(binary, (char) => char.charCodeAt(0));
}

/** A blocky stand-in figure coloured from the boxer's sample palette. */
function drawFigure(ctx: CanvasRenderingContext2D, palette: Rgb[], pose: number, scale: number): void {
  const px = (x: number, y: number, w: number, h: number, color: Rgb) => {
    ctx.fillStyle = css(color);
    ctx.fillRect(x * scale, y * scale, w * scale, h * scale);
  };
  const lean = (pose % 3) - 1;
  px(13 + lean, 4, 6, 6, palette[4]); // head
  px(13 + lean, 3, 6, 2, palette[1]); // hair
  px(11, 10, 10, 9, palette[3]); // body
  px(8 + lean, 11, 3, 6, palette[8]); // glove
  px(21 + lean, 9, 3, 6, palette[8]); // glove
  px(11, 19, 10, 5, palette[5]); // trunks
  px(11, 23, 10, 1, palette[6]);
  px(12, 24, 3, 6, palette[3]); // legs
  px(17, 24, 3, 6, palette[3]);
  px(11, 29, 4, 2, palette[7]); // shoes
  px(17, 29, 4, 2, palette[7]);
}

const boxerRecord = (name: string) => {
  const key = keyOf(name);
  const asset = (subtype: string, offset: number, size: number, category = "Graphics") => ({
    file: `${category}/${key}_${subtype}.bin`,
    filename: `${key}_${subtype}.bin`,
    category,
    subtype,
    size,
    start_snes: "$000000",
    end_snes: "$000000",
    start_pc: `0x${(offset + BOXERS.indexOf(name) * 0x1000).toString(16).toUpperCase()}`,
    end_pc: "0x0",
    shared_with: [],
  });
  return {
    name,
    key,
    reference_sheet: "",
    palette_files: [asset("palette", 0x100, 64, "Palettes")],
    icon_files: [asset("icon", 0x200, 512)],
    portrait_files: [],
    large_portrait_files: [asset("large_portrait", 0x400, 2048, "Graphics/Compressed")],
    unique_sprite_bins: [],
    shared_sprite_bins: [],
    other_files: [],
  };
};

const boxerByPaletteOffset = (offset: string): string | undefined =>
  BOXERS.map(keyOf).find((key) => {
    const record = boxerRecord(BOXERS.find((name) => keyOf(name) === key)!);
    return record.palette_files[0].start_pc === offset;
  });

const summaries = (stack: Edit[]) =>
  [...stack].reverse().map((edit, index) => ({
    id: index,
    action_type: "Transaction",
    description: edit.label,
    pc_offset: null,
    timestamp: new Date().toISOString(),
  }));

type Args = Record<string, unknown>;

function handle(cmd: string, args: Args): unknown {
  switch (cmd) {
    // --- window/event/dialog plumbing -----------------------------------
    case "plugin:event|listen":
      return 0;
    case "plugin:event|unlisten":
    case "plugin:event|emit":
      return null;
    case "plugin:dialog|open":
      return SAMPLE_PATH;
    case "plugin:dialog|save":
      return "C:\\Preview\\Saved File";

    // --- ROM session ----------------------------------------------------
    case "get_rom_sha1":
      if (!romLoaded) throw "No ROM loaded";
      return SAMPLE_SHA1;
    case "is_rom_loaded":
      return romLoaded;
    case "get_rom_path":
      return romLoaded ? SAMPLE_PATH : null;
    case "detect_rom_region":
      return {
        success: true,
        region: "Usa",
        display_name: "Sample Game (Preview)",
        is_supported: true,
        sha1: SAMPLE_SHA1,
        error_message: null,
      };
    case "get_supported_regions":
      return [];
    case "open_rom":
      romLoaded = true;
      undoStack.length = 0;
      redoStack.length = 0;
      return SAMPLE_SHA1;
    case "get_boxers":
      return romLoaded ? BOXERS.map(boxerRecord) : [];
    case "get_boxer":
      return boxerRecord(BOXERS.find((name) => keyOf(name) === args.key) ?? BOXERS[0]);
    case "get_pending_writes":
      return undoStack.map((_, index) => `0x${(0x100 + index).toString(16)}`);
    case "get_loaded_rom_image":
      return [];

    // --- palettes -------------------------------------------------------
    case "get_palette":
      return palettes.get(boxerByPaletteOffset(String(args.pcOffset)) ?? "") ?? samplePalette(0);
    case "get_original_palette":
      return originals.get(boxerByPaletteOffset(String(args.pcOffset)) ?? "") ?? samplePalette(0);
    case "apply_palette_colors": {
      const boxer = boxerByPaletteOffset(String(args.pcOffset));
      if (!boxer) throw "Unknown palette";
      const before = palettes.get(boxer)!;
      const after = args.colors as Rgb[];
      if (JSON.stringify(before) === JSON.stringify(after)) return false;
      palettes.set(boxer, after);
      undoStack.push({ label: String(args.label ?? "Recolor"), before, after, boxer });
      redoStack.length = 0;
      return true;
    }

    // --- history --------------------------------------------------------
    case "can_undo":
      return undoStack.length > 0;
    case "can_redo":
      return redoStack.length > 0;
    case "get_undo_stack":
      return summaries(undoStack);
    case "get_redo_stack":
      return summaries(redoStack);
    case "undo": {
      const edit = undoStack.pop();
      if (!edit) throw "Nothing to undo";
      palettes.set(edit.boxer, edit.before);
      redoStack.push(edit);
      return null;
    }
    case "redo": {
      const edit = redoStack.pop();
      if (!edit) throw "Nothing to redo";
      palettes.set(edit.boxer, edit.after);
      undoStack.push(edit);
      return null;
    }

    // --- pictures -------------------------------------------------------
    case "get_runtime_theme_assets": {
      const key = String(args.boxerKey ?? keyOf(BOXERS[0]));
      const palette = palettes.get(key) ?? samplePalette(0);
      const icon = pngBytes(32, (ctx) => {
        ctx.fillStyle = css(palette[5]);
        ctx.fillRect(0, 0, 32, 32);
        ctx.translate(-16, -8);
        drawFigure(ctx, palette, 0, 2);
      });
      return {
        boxer_key: key,
        boxer_name: BOXERS.find((name) => keyOf(name) === key) ?? BOXERS[0],
        palette: palette.slice(0, 16),
        icon_png: icon,
        portrait_png: null,
      };
    }
    case "render_asset_preview": {
      if (String(args.category).includes("Compressed")) throw "Sample: this picture cannot be read";
      const key = boxerByPaletteOffset(String(args.palettePcOffset)) ?? keyOf(BOXERS[0]);
      const palette = palettes.get(key) ?? samplePalette(0);
      return pngBytes(32, (ctx) => {
        ctx.fillStyle = css(palette[5]);
        ctx.fillRect(0, 0, 32, 32);
        ctx.translate(-16, -8);
        drawFigure(ctx, palette, 0, 2);
      });
    }
    case "get_fighter_list":
      return BOXERS.map((name, id) => ({ id, name, header_addr: 0, palette_addr: 0, pose_table_addr: 0 }));
    case "get_fighter_poses":
      return Array.from({ length: 6 }, (_, index) => ({
        index,
        tileset1_id: 0,
        tileset2_id: 0,
        palette_id: 0,
        data_addr: 0x8000 + index,
      }));
    case "render_fighter_pose": {
      const palette = palettes.get(keyOf(BOXERS[Number(args.fighterId) || 0])) ?? samplePalette(0);
      return pngBytes(256, (ctx) => drawFigure(ctx, palette, Number(args.poseId) || 0, 8));
    }

    // --- play game ------------------------------------------------------
    case "get_emulator_settings":
      return {
        emulator_path: emulatorPath,
        emulator_type: "snes9x",
        auto_save_before_launch: true,
        command_line_args: "",
        jump_to_selected_boxer: true,
        default_round: 1,
        save_state_dir: null,
      };
    case "set_emulator_settings":
      emulatorPath = String((args.settings as { emulator_path?: string })?.emulator_path ?? "");
      return null;
    case "verify_emulator":
      return { valid: emulatorPath !== "", message: emulatorPath ? "Emulator found" : "No emulator configured" };
    case "find_installed_emulators":
      return [
        {
          path: "C:\\Preview\\Emulators\\snes9x\\snes9x-x64.exe",
          emulator_type: "snes9x",
          name: "Snes9x",
          folder: "C:\\Preview\\Emulators\\snes9x",
        },
      ];
    case "test_in_emulator":
    case "save_rom_as":
      return null;
    case "stamp_photo_on_pose":
      return { pixels_written: 600, pixels_skipped: 40, tiles_changed: 14, other_poses_changed: [1, 2, 3], also_affects: [] };
    case "autosave_session":
    case "restore_autosave":
      return 0;
    case "export_ips_patch":
    case "export_bps_patch":
      return undoStack.length;

    // --- projects, settings, help --------------------------------------
    case "get_current_project":
    case "get_current_project_path":
    case "load_theme_settings":
      return null;
    case "save_theme_settings":
      return null;
    case "should_auto_check":
      return false;
    case "get_current_version":
      return "preview";
    case "get_external_tools":
    case "get_tool_categories":
    case "get_help_articles":
    case "get_save_state_slots":
      return [];

    default:
      throw `"${cmd}" is not available in the UI preview`;
  }
}

/** Install the stand-in backend. Call before the app renders. */
export function installPreviewBackend(): void {
  mockWindows("main");
  mockIPC((cmd, payload) => handle(cmd, (payload ?? {}) as Args), { shouldMockEvents: true });
  (globalThis as { isTauri?: boolean }).isTauri = true;
  console.info("UI preview backend installed: all data on screen is made-up sample data.");
}
