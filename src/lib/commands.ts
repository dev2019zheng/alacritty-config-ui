import { invoke } from "@tauri-apps/api/core";

import type {
  Colors,
  ConfigStateDto,
  EditorConfig,
  HexColor,
  ThemeDocumentConfig,
  ThemeEntryDto,
} from "../types/config";

export const isTauriRuntime =
  typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

const demoRootPath = "/Users/demo/.config/alacritty/alacritty.toml";
let mockState = createMockState(demoRootPath);
let mockThemes = createMockThemes(mockState.rootPath);

export function loadDefaultConfig() {
  return call<ConfigStateDto>("load_default_config");
}

export function loadConfig(path: string) {
  return call<ConfigStateDto>("load_config", { path });
}

export function reloadConfig() {
  return call<ConfigStateDto>("reload_config");
}

export function saveConfig() {
  return call<ConfigStateDto>("save_config");
}

export function updateConfig(merged: EditorConfig) {
  return call<ConfigStateDto>("update_config", { merged });
}

export function setBaseImport(path: string) {
  return call<ConfigStateDto>("set_base_import", { path });
}

export function setThemeImport(path: string) {
  return call<ConfigStateDto>("set_theme_import", { path });
}

export function listThemes() {
  return call<ThemeEntryDto[]>("list_themes");
}

export function applyThemeEntry(entry: ThemeEntryDto) {
  return call<ConfigStateDto>("apply_theme_entry", { entry });
}

export function listSystemFonts() {
  return call<string[]>("list_system_fonts");
}

async function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  if (isTauriRuntime) {
    return invoke<T>(command, args);
  }

  return mockInvoke<T>(command, args);
}

async function mockInvoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  switch (command) {
    case "load_default_config": {
      mockState = createMockState(demoRootPath);
      mockThemes = createMockThemes(mockState.rootPath);
      return mockState as T;
    }
    case "load_config": {
      const path = String(args?.path ?? demoRootPath);
      mockState = createMockState(path);
      mockThemes = createMockThemes(mockState.rootPath);
      return mockState as T;
    }
    case "reload_config": {
      return mockState as T;
    }
    case "save_config": {
      mockState = { ...mockState, dirty: false };
      return mockState as T;
    }
    case "update_config": {
      const merged = structuredClone(args?.merged as EditorConfig);
      merged.general.import = buildImports(mockState.rootPath, mockState.basePath, mockState.activeThemePath);
      mockState = { ...mockState, merged, dirty: true };
      return mockState as T;
    }
    case "set_base_import": {
      const basePath = String(args?.path ?? "");
      mockState = {
        ...mockState,
        basePath,
        dirty: true,
        merged: {
          ...mockState.merged,
          general: {
            ...mockState.merged.general,
            import: buildImports(mockState.rootPath, basePath, mockState.activeThemePath),
          },
        },
      };
      return mockState as T;
    }
    case "set_theme_import": {
      const activeThemePath = String(args?.path ?? "");
      mockState = {
        ...mockState,
        activeThemePath,
        dirty: true,
        merged: {
          ...mockState.merged,
          general: {
            ...mockState.merged.general,
            import: buildImports(mockState.rootPath, mockState.basePath, activeThemePath),
          },
        },
      };
      return mockState as T;
    }
    case "list_themes": {
      return mockThemes as T;
    }
    case "apply_theme_entry": {
      const entry = structuredClone(args?.entry as ThemeEntryDto);
      const nextMerged = structuredClone(mockState.merged);
      applyThemeFragment(nextMerged, entry.fragment);
      const activeThemePath = entry.source === "BundledPreset" || entry.source === "LocalPreset"
        ? `${dirname(mockState.rootPath)}/themes/custom/${slugify(entry.name)}.toml`
        : entry.path;
      nextMerged.general.import = buildImports(mockState.rootPath, mockState.basePath, activeThemePath);
      mockState = {
        ...mockState,
        merged: nextMerged,
        activeThemePath,
        dirty: false,
      };
      mockThemes = createMockThemes(mockState.rootPath);
      return mockState as T;
    }
    case "list_system_fonts": {
      return [
        "Berkeley Mono",
        "Fira Code",
        "JetBrains Mono",
        "Menlo",
        "Monaco",
        "Operator Mono",
        "SF Mono",
      ] as T;
    }
    default:
      throw new Error(`Unsupported mock command: ${command}`);
  }
}

function createMockState(rootPath: string): ConfigStateDto {
  const basePath = `${dirname(rootPath)}/base.toml`;
  const activeThemePath = `${dirname(rootPath)}/themes/custom/tokyo-night.toml`;
  const merged = createDefaultEditorConfig();
  merged.general.import = buildImports(rootPath, basePath, activeThemePath);

  return {
    merged,
    rootPath,
    basePath,
    activeThemePath,
    dirty: false,
  };
}

function createMockThemes(rootPath: string): ThemeEntryDto[] {
  const rootDir = dirname(rootPath);

  return [
    createThemeEntry({
      name: "Tokyo Night",
      path: `${rootDir}/themes/themes/tokyo-night.toml`,
      source: "BundledPreset",
      background: "#0F111A",
      foreground: "#C5D1EB",
      accent: "#7AA2F7",
      cyan: "#7FDBCA",
    }),
    createThemeEntry({
      name: "Catppuccin Mocha",
      path: `${rootDir}/themes/themes/catppuccin-mocha.toml`,
      source: "BundledPreset",
      background: "#1E1E2E",
      foreground: "#CDD6F4",
      accent: "#89B4FA",
      cyan: "#94E2D5",
    }),
    createThemeEntry({
      name: "Solarized Dark",
      path: `${rootDir}/themes/themes/solarized-dark.toml`,
      source: "LocalPreset",
      background: "#002B36",
      foreground: "#93A1A1",
      accent: "#268BD2",
      cyan: "#2AA198",
    }),
    createThemeEntry({
      name: "Midnight Spruce",
      path: `${rootDir}/themes/custom/midnight-spruce.toml`,
      source: "CustomTheme",
      background: "#09131B",
      foreground: "#D5E5EF",
      accent: "#6EE7B7",
      cyan: "#7DD3FC",
    }),
  ];
}

function createThemeEntry(params: {
  name: string;
  path: string;
  source: ThemeEntryDto["source"];
  background: HexColor;
  foreground: HexColor;
  accent: HexColor;
  cyan: HexColor;
}): ThemeEntryDto {
  const colors = createDefaultColors();
  colors.primary.background = params.background;
  colors.primary.foreground = params.foreground;
  colors.primary.dim_foreground = adjustBrightness(params.foreground, -22);
  colors.primary.bright_foreground = adjustBrightness(params.foreground, 14);
  colors.cursor.cursor = params.accent;
  colors.selection.background = adjustBrightness(params.background, 18);
  colors.normal.blue = params.accent;
  colors.bright.blue = adjustBrightness(params.accent, 18);
  colors.normal.cyan = params.cyan;
  colors.bright.cyan = adjustBrightness(params.cyan, 14);

  return {
    name: params.name,
    path: params.path,
    source: params.source,
    fragment: {
      window: null,
      font: null,
      colors,
    },
  };
}

function createDefaultEditorConfig(): EditorConfig {
  return {
    general: {
      import: [],
      live_config_reload: true,
    },
    scrolling: {
      history: 10000,
      multiplier: 3,
    },
    cursor: {
      style: {
        shape: "Block",
        blinking: "Off",
      },
      vi_mode_style: "None",
      unfocused_hollow: true,
      thickness: 0.15,
    },
    selection: {
      semantic_escape_chars: ',│`|:"\' ()[]{}<>\t',
      save_to_clipboard: false,
    },
    font: {
      normal: { family: "Menlo", style: "Regular" },
      bold: { family: "Menlo", style: "Bold" },
      italic: { family: "Menlo", style: "Italic" },
      bold_italic: { family: "Menlo", style: "Bold Italic" },
      size: 11.25,
      builtin_box_drawing: true,
    },
    window: {
      padding: { x: 10, y: 10 },
      dynamic_padding: false,
      decorations: "Full",
      opacity: 0.96,
      blur: false,
      option_as_alt: "None",
      dynamic_title: true,
    },
    colors: createDefaultColors(),
  };
}

function createDefaultColors(): Colors {
  return {
    draw_bold_text_with_bright_colors: true,
    primary: {
      background: "#0F111A",
      foreground: "#C5D1EB",
      dim_foreground: "#8A93A8",
      bright_foreground: "#E6EDF7",
    },
    cursor: {
      text: "CellBackground",
      cursor: "#89DDFF",
    },
    selection: {
      text: "CellForeground",
      background: "#273244",
    },
    normal: {
      black: "#1B1F2A",
      red: "#F07178",
      green: "#A6DA95",
      yellow: "#EBCB8B",
      blue: "#7AA2F7",
      magenta: "#C099FF",
      cyan: "#7FDBCA",
      white: "#C5D1EB",
    },
    bright: {
      black: "#3A4154",
      red: "#FF8F97",
      green: "#B8F0B0",
      yellow: "#F5D7A1",
      blue: "#8AB4FF",
      magenta: "#D2A6FF",
      cyan: "#94F0E0",
      white: "#E6EDF7",
    },
  };
}

function applyThemeFragment(merged: EditorConfig, fragment: ThemeDocumentConfig) {
  if (fragment.window) {
    merged.window = structuredClone(fragment.window);
  }
  if (fragment.font) {
    merged.font = {
      ...merged.font,
      ...structuredClone(fragment.font),
      builtin_box_drawing: merged.font.builtin_box_drawing,
    };
  }
  if (fragment.colors) {
    merged.colors = structuredClone(fragment.colors);
  }
}

function buildImports(rootPath: string, basePath: string | null, activeThemePath: string | null) {
  const imports: string[] = [];

  if (basePath) {
    imports.push(toImportPath(rootPath, basePath));
  }
  if (activeThemePath) {
    imports.push(toImportPath(rootPath, activeThemePath));
  }

  return imports;
}

function toImportPath(rootPath: string, targetPath: string) {
  const rootDir = dirname(rootPath);
  const prefix = `${rootDir}/`;
  return targetPath.startsWith(prefix) ? targetPath.slice(prefix.length) : targetPath;
}

function dirname(path: string) {
  const lastSlash = path.lastIndexOf("/");
  return lastSlash >= 0 ? path.slice(0, lastSlash) : ".";
}

function slugify(value: string) {
  return value
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "");
}

function adjustBrightness(color: HexColor, amount: number): HexColor {
  const [red, green, blue] = [1, 3, 5].map((offset) =>
    Number.parseInt(color.slice(offset, offset + 2), 16),
  );

  return (`#${[red, green, blue]
    .map((channel) => Math.max(0, Math.min(255, channel + amount)).toString(16).padStart(2, "0"))
    .join("")
    .toUpperCase()}`) as HexColor;
}
