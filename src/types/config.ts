export type HexColor = `#${string}`;
export type CellColor = "CellBackground" | "CellForeground" | HexColor;

export interface GeneralConfig {
  import: string[];
  live_config_reload: boolean;
}

export interface ScrollingConfig {
  history: number;
  multiplier: number;
}

export type CursorShape = "Block" | "Underline" | "Beam";
export type CursorBlinking = "Never" | "Off" | "On" | "Always";

export interface CursorStyleConfig {
  shape: CursorShape;
  blinking: CursorBlinking;
}

export type ViModeStyle = "None" | CursorStyleConfig;

export interface CursorConfig {
  style: CursorStyleConfig;
  vi_mode_style: ViModeStyle;
  unfocused_hollow: boolean;
  thickness: number;
}

export interface SelectionConfig {
  semantic_escape_chars: string;
  save_to_clipboard: boolean;
}

export interface FontFace {
  family: string;
  style: string;
}

export interface FontConfig {
  normal: FontFace;
  bold: FontFace;
  italic: FontFace;
  bold_italic: FontFace;
  size: number;
  builtin_box_drawing: boolean;
}

export interface ThemeFontConfig {
  normal: FontFace;
  bold: FontFace;
  italic: FontFace;
  bold_italic: FontFace;
  size: number;
}

export interface Padding {
  x: number;
  y: number;
}

export type Decorations = "Full" | "Transparent" | "Buttonless" | "None";
export type OptionAsAlt = "OnlyLeft" | "OnlyRight" | "Both" | "None";

export interface WindowConfig {
  padding: Padding;
  dynamic_padding: boolean;
  decorations: Decorations;
  opacity: number;
  blur: boolean;
  option_as_alt: OptionAsAlt;
  dynamic_title: boolean;
}

export interface PrimaryColors {
  background: HexColor;
  foreground: HexColor;
  dim_foreground: HexColor | null;
  bright_foreground: HexColor | null;
}

export interface CursorColors {
  text: CellColor;
  cursor: CellColor;
}

export interface SelectionColors {
  text: CellColor;
  background: CellColor;
}

export interface NamedColors {
  black: HexColor;
  red: HexColor;
  green: HexColor;
  yellow: HexColor;
  blue: HexColor;
  magenta: HexColor;
  cyan: HexColor;
  white: HexColor;
}

export interface Colors {
  draw_bold_text_with_bright_colors: boolean;
  primary: PrimaryColors;
  cursor: CursorColors;
  selection: SelectionColors;
  normal: NamedColors;
  bright: NamedColors;
}

export interface EditorConfig {
  general: GeneralConfig;
  scrolling: ScrollingConfig;
  cursor: CursorConfig;
  selection: SelectionConfig;
  font: FontConfig;
  window: WindowConfig;
  colors: Colors;
}

export interface ThemeDocumentConfig {
  window: WindowConfig | null;
  font: ThemeFontConfig | null;
  colors: Colors | null;
}

export type ThemeSourceDto = "BundledPreset" | "LocalPreset" | "LocalTheme" | "CustomTheme";

export interface ThemeEntryDto {
  name: string;
  path: string;
  source: ThemeSourceDto;
  fragment: ThemeDocumentConfig;
}

export interface ConfigStateDto {
  merged: EditorConfig;
  rootPath: string;
  basePath: string | null;
  activeThemePath: string | null;
  dirty: boolean;
}
