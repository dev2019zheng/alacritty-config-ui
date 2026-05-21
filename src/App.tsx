import { useEffect, useMemo, useRef, useState } from "react";
import {
  AlertCircle,
  CheckCircle2,
  FolderOpen,
  Monitor,
  Palette,
  RefreshCw,
  Save,
  Search,
  SlidersHorizontal,
  TerminalSquare,
  Type,
  type LucideIcon,
} from "lucide-react";

import { isTauriRuntime } from "./lib/commands";
import { cn } from "./lib/utils";
import { useConfigStore, type PanelId } from "./store/config";
import type {
  CellColor,
  ConfigStateDto,
  CursorShape,
  EditorConfig,
  HexColor,
  ThemeEntryDto,
  ThemeSourceDto,
} from "./types/config";

const panels: Array<{
  id: PanelId;
  label: string;
  description: string;
  icon: LucideIcon;
}> = [
  { id: "theme", label: "Themes", description: "预览并应用主题", icon: Palette },
  { id: "appearance", label: "Appearance", description: "窗口与配色", icon: Monitor },
  { id: "text", label: "Text", description: "字体与字号", icon: Type },
  { id: "terminal", label: "Terminal", description: "导入、光标与滚动", icon: SlidersHorizontal },
];

const sourceLabels: Record<ThemeSourceDto, string> = {
  BundledPreset: "Bundled",
  LocalPreset: "Local preset",
  LocalTheme: "Local theme",
  CustomTheme: "Custom",
};

const sourceClasses: Record<ThemeSourceDto, string> = {
  BundledPreset: "border-blue-400/20 bg-blue-400/10 text-blue-100",
  LocalPreset: "border-violet-400/20 bg-violet-400/10 text-violet-100",
  LocalTheme: "border-amber-400/20 bg-amber-400/10 text-amber-100",
  CustomTheme: "border-emerald-400/20 bg-emerald-400/10 text-emerald-100",
};

const decorationOptions = ["Full", "Transparent", "Buttonless", "None"] as const;
const optionAsAltOptions = ["None", "OnlyLeft", "OnlyRight", "Both"] as const;
const cursorShapeOptions = ["Block", "Underline", "Beam"] as const;
const cursorBlinkingOptions = ["Never", "Off", "On", "Always"] as const;

type ChromeTheme = "graphite" | "paper";

export default function App() {
  const config = useConfigStore((state) => state.config);
  const themes = useConfigStore((state) => state.themes);
  const fonts = useConfigStore((state) => state.fonts);
  const loading = useConfigStore((state) => state.loading);
  const saving = useConfigStore((state) => state.saving);
  const error = useConfigStore((state) => state.error);
  const selectedPanel = useConfigStore((state) => state.selectedPanel);
  const bootstrap = useConfigStore((state) => state.bootstrap);
  const loadConfig = useConfigStore((state) => state.loadConfig);
  const reload = useConfigStore((state) => state.reload);
  const save = useConfigStore((state) => state.save);
  const patchConfig = useConfigStore((state) => state.patchConfig);
  const applyTheme = useConfigStore((state) => state.applyTheme);
  const selectPanel = useConfigStore((state) => state.selectPanel);
  const setError = useConfigStore((state) => state.setError);

  const [themeQuery, setThemeQuery] = useState("");
  const [chromeTheme, setChromeTheme] = useState<ChromeTheme>("graphite");
  const bootstrapped = useRef(false);

  useEffect(() => {
    if (bootstrapped.current) {
      return;
    }

    bootstrapped.current = true;
    void bootstrap();
  }, [bootstrap]);

  const filteredThemes = useMemo(() => {
    const query = themeQuery.trim().toLowerCase();
    if (!query) {
      return themes;
    }

    return themes.filter((entry) => {
      const haystack = [entry.name, entry.source, entry.path].join(" ").toLowerCase();
      return haystack.includes(query);
    });
  }, [themeQuery, themes]);

  const fontOptions = useMemo(() => {
    if (!config) {
      return fonts;
    }

    const seeded = new Set(fonts);
    seeded.add(config.merged.font.normal.family);
    seeded.add(config.merged.font.bold.family);
    seeded.add(config.merged.font.italic.family);
    seeded.add(config.merged.font.bold_italic.family);
    return Array.from(seeded).sort((left, right) => left.localeCompare(right));
  }, [config, fonts]);

  const themeCounts = useMemo(
    () => ({
      total: themes.length,
      bundled: themes.filter((entry) => entry.source === "BundledPreset").length,
      local: themes.filter((entry) => entry.source !== "BundledPreset").length,
      custom: themes.filter((entry) => entry.source === "CustomTheme").length,
    }),
    [themes],
  );

  const openConfig = async () => {
    if (!isTauriRuntime) {
      setError("当前是浏览器预览模式，Open Config 需要在 Tauri 窗口里执行。");
      return;
    }

    try {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const selection = await open({
        multiple: false,
        directory: false,
        filters: [{ name: "TOML", extensions: ["toml"] }],
      });

      if (typeof selection === "string") {
        await loadConfig(selection);
      }
    } catch (openError) {
      setError(toMessage(openError));
    }
  };

  const updateLiveReload = (checked: boolean) => {
    void patchConfig((draft) => {
      draft.general.live_config_reload = checked;
    });
  };

  const updateFontFamily = (
    face: "normal" | "bold" | "italic" | "bold_italic",
    family: string,
  ) => {
    void patchConfig((draft) => {
      draft.font[face].family = family;
    });
  };

  const updateFontSize = (size: number) => {
    void patchConfig((draft) => {
      draft.font.size = round(size, 2);
    });
  };

  const updateBuiltinBoxDrawing = (checked: boolean) => {
    void patchConfig((draft) => {
      draft.font.builtin_box_drawing = checked;
    });
  };

  const updatePrimaryColor = (key: "background" | "foreground", value: string) => {
    const normalized = normalizeHex(value);
    if (!normalized) {
      return;
    }

    void patchConfig((draft) => {
      draft.colors.primary[key] = normalized;
    });
  };

  const updateDrawBoldBright = (checked: boolean) => {
    void patchConfig((draft) => {
      draft.colors.draw_bold_text_with_bright_colors = checked;
    });
  };

  const updateWindowOpacity = (opacity: number) => {
    void patchConfig((draft) => {
      draft.window.opacity = round(opacity, 2);
    });
  };

  const updateWindowPadding = (axis: "x" | "y", value: number) => {
    void patchConfig((draft) => {
      draft.window.padding[axis] = Math.max(0, Math.round(value));
    });
  };

  const updateWindowToggle = (
    key: "dynamic_padding" | "blur" | "dynamic_title",
    checked: boolean,
  ) => {
    void patchConfig((draft) => {
      draft.window[key] = checked;
    });
  };

  const updateWindowEnum = (key: "decorations" | "option_as_alt", value: string) => {
    void patchConfig((draft) => {
      if (key === "decorations") {
        draft.window.decorations = value as EditorConfig["window"]["decorations"];
      } else {
        draft.window.option_as_alt = value as EditorConfig["window"]["option_as_alt"];
      }
    });
  };

  const updateCursorShape = (shape: string) => {
    void patchConfig((draft) => {
      draft.cursor.style.shape = shape as EditorConfig["cursor"]["style"]["shape"];
      if (draft.cursor.vi_mode_style !== "None") {
        draft.cursor.vi_mode_style = {
          ...draft.cursor.vi_mode_style,
          shape: shape as EditorConfig["cursor"]["style"]["shape"],
        };
      }
    });
  };

  const updateCursorBlinking = (blinking: string) => {
    void patchConfig((draft) => {
      draft.cursor.style.blinking = blinking as EditorConfig["cursor"]["style"]["blinking"];
      if (draft.cursor.vi_mode_style !== "None") {
        draft.cursor.vi_mode_style = {
          ...draft.cursor.vi_mode_style,
          blinking: blinking as EditorConfig["cursor"]["style"]["blinking"],
        };
      }
    });
  };

  const updateCursorThickness = (value: number) => {
    void patchConfig((draft) => {
      draft.cursor.thickness = round(value, 2);
    });
  };

  const updateCursorToggle = (checked: boolean) => {
    void patchConfig((draft) => {
      draft.cursor.unfocused_hollow = checked;
    });
  };

  const updateViModeStyle = (mode: "None" | "FollowCursor") => {
    void patchConfig((draft) => {
      draft.cursor.vi_mode_style = mode === "None" ? "None" : { ...draft.cursor.style };
    });
  };

  const updateSelectionToggle = (checked: boolean) => {
    void patchConfig((draft) => {
      draft.selection.save_to_clipboard = checked;
    });
  };

  const updateSelectionChars = (value: string) => {
    void patchConfig((draft) => {
      draft.selection.semantic_escape_chars = value;
    });
  };

  const updateScrolling = (key: "history" | "multiplier", value: number) => {
    void patchConfig((draft) => {
      draft.scrolling[key] = Math.max(0, Math.round(value));
    });
  };

  const panelContent = (() => {
    if (loading && !config) {
      return <LoadingState />;
    }

    if (!config) {
      return <EmptyState onRetry={() => void bootstrap()} />;
    }

    switch (selectedPanel) {
      case "theme":
        return (
          <div className="space-y-6">
            <SectionHeader title="Themes" />

            <div className="grid gap-3 xl:grid-cols-[minmax(0,1fr)_auto] xl:items-center">
              <label className="relative min-w-[280px] flex-1 max-w-xl">
                <Search className="pointer-events-none absolute left-4 top-1/2 h-4 w-4 -translate-y-1/2 text-[var(--muted)]" />
                <input
                  value={themeQuery}
                  onChange={(event) => setThemeQuery(event.target.value)}
                  placeholder="搜索主题名、来源或路径"
                  className="h-11 w-full rounded-2xl border border-[var(--border-strong)] bg-[var(--field)] py-2 pl-11 pr-4 text-sm text-[var(--foreground)] outline-none transition placeholder:text-[var(--muted)] focus:border-[var(--accent)] focus:bg-[var(--field-focus)]"
                />
              </label>
              <div className="flex flex-wrap items-center gap-2">
                <SummaryPill label="All" value={themeCounts.total} />
                <SummaryPill label="Bundled" value={themeCounts.bundled} />
                <SummaryPill label="Local" value={themeCounts.local} />
                <SummaryPill label="Custom" value={themeCounts.custom} />
              </div>
            </div>

            <div className="grid gap-5 md:grid-cols-2 2xl:grid-cols-3">
              {filteredThemes.map((entry) => (
                <ThemeCard
                  key={`${entry.source}:${entry.path}`}
                  entry={entry}
                  active={isThemeActive(entry, config.activeThemePath)}
                  onApply={() => void applyTheme(entry)}
                />
              ))}
            </div>

            {filteredThemes.length === 0 && (
              <div className="rounded-[22px] border border-dashed border-[var(--border)] bg-[var(--card)] px-5 py-8 text-sm text-[var(--muted)]">
                没有找到匹配的主题，换个关键词再试。
              </div>
            )}
          </div>
        );
      case "appearance":
        return (
          <div className="space-y-6">
            <SectionHeader title="Appearance" />

            <div className="grid gap-5 xl:grid-cols-[minmax(0,1.02fr)_minmax(0,0.98fr)]">
              <SettingSection title="Palette" className="xl:min-h-full">
                <ColorInputRow
                  label="Background"
                  note="终端底色"
                  value={config.merged.colors.primary.background}
                  onChange={(value) => updatePrimaryColor("background", value)}
                />
                <ColorInputRow
                  label="Foreground"
                  note="正文前景色"
                  value={config.merged.colors.primary.foreground}
                  onChange={(value) => updatePrimaryColor("foreground", value)}
                />
                <ToggleRow
                  label="Brighten bold text"
                  note="粗体文本优先使用 bright 颜色表"
                  checked={config.merged.colors.draw_bold_text_with_bright_colors}
                  onChange={updateDrawBoldBright}
                />
                <SettingRow label="Palette cards" note="Normal / Bright" alignTop>
                  <div className="grid gap-3 xl:grid-cols-2">
                    <PaletteCard title="Normal palette" colors={config.merged.colors.normal} />
                    <PaletteCard title="Bright palette" colors={config.merged.colors.bright} />
                  </div>
                </SettingRow>
              </SettingSection>

              <SettingSection title="Window" className="xl:min-h-full">
                <RangeRow
                  label="Opacity"
                  note="窗口透明度"
                  min={0.2}
                  max={1}
                  step={0.01}
                  value={config.merged.window.opacity}
                  display={`${Math.round(config.merged.window.opacity * 100)}%`}
                  onChange={updateWindowOpacity}
                />
                <SettingRow label="Padding" note="X / Y" alignTop>
                  <div className="grid w-full max-w-[320px] gap-2.5 sm:grid-cols-2">
                    <CompactNumberField
                      label="X"
                      value={config.merged.window.padding.x}
                      min={0}
                      onChange={(value) => updateWindowPadding("x", value)}
                    />
                    <CompactNumberField
                      label="Y"
                      value={config.merged.window.padding.y}
                      min={0}
                      onChange={(value) => updateWindowPadding("y", value)}
                    />
                  </div>
                </SettingRow>
                <SelectRow
                  label="Decorations"
                  note="标题栏与边框样式"
                  value={config.merged.window.decorations}
                  options={Array.from(decorationOptions)}
                  onChange={(value) => updateWindowEnum("decorations", value)}
                />
                <SelectRow
                  label="Option as Alt"
                  note="macOS Option 键映射"
                  value={config.merged.window.option_as_alt}
                  options={Array.from(optionAsAltOptions)}
                  onChange={(value) => updateWindowEnum("option_as_alt", value)}
                />
                <SettingRow label="Flags" note="Blur / padding / title" alignTop>
                  <div className="grid gap-2.5 sm:grid-cols-3">
                    <MiniToggleCard
                      label="Blur"
                      checked={config.merged.window.blur}
                      onChange={(checked) => updateWindowToggle("blur", checked)}
                    />
                    <MiniToggleCard
                      label="Dynamic padding"
                      checked={config.merged.window.dynamic_padding}
                      onChange={(checked) => updateWindowToggle("dynamic_padding", checked)}
                    />
                    <MiniToggleCard
                      label="Dynamic title"
                      checked={config.merged.window.dynamic_title}
                      onChange={(checked) => updateWindowToggle("dynamic_title", checked)}
                    />
                  </div>
                </SettingRow>
              </SettingSection>
            </div>
          </div>
        );
      case "text":
        return (
          <div className="space-y-6">
            <SectionHeader title="Text" />

            <SettingSection title="Typography">
              <RangeRow
                label="Font size"
                note="终端主字号"
                min={8}
                max={22}
                step={0.25}
                value={config.merged.font.size}
                display={`${config.merged.font.size.toFixed(2)} pt`}
                onChange={updateFontSize}
              />
              <SettingRow label="Font families" note="Normal / Bold / Italic / Bold italic" alignTop>
                <div className="grid gap-3 sm:grid-cols-2">
                  <FontFamilyCard
                    label="Normal"
                    value={config.merged.font.normal.family}
                    options={fontOptions}
                    onChange={(value) => updateFontFamily("normal", value)}
                  />
                  <FontFamilyCard
                    label="Bold"
                    value={config.merged.font.bold.family}
                    options={fontOptions}
                    onChange={(value) => updateFontFamily("bold", value)}
                  />
                  <FontFamilyCard
                    label="Italic"
                    value={config.merged.font.italic.family}
                    options={fontOptions}
                    onChange={(value) => updateFontFamily("italic", value)}
                  />
                  <FontFamilyCard
                    label="Bold italic"
                    value={config.merged.font.bold_italic.family}
                    options={fontOptions}
                    onChange={(value) => updateFontFamily("bold_italic", value)}
                  />
                </div>
              </SettingRow>
              <ToggleRow
                label="Builtin box drawing"
                note="字体缺少 box drawing 时使用内建字形"
                checked={config.merged.font.builtin_box_drawing}
                onChange={updateBuiltinBoxDrawing}
              />
              <SettingRow label="Resolved styles" note="Current styles" alignTop>
                <div className="grid gap-3 sm:grid-cols-2 xl:grid-cols-4">
                  <InfoTile label="Normal" value={config.merged.font.normal.style} />
                  <InfoTile label="Bold" value={config.merged.font.bold.style} />
                  <InfoTile label="Italic" value={config.merged.font.italic.style} />
                  <InfoTile label="Bold italic" value={config.merged.font.bold_italic.style} />
                </div>
              </SettingRow>
            </SettingSection>
          </div>
        );
      case "terminal":
        return (
          <div className="space-y-6">
            <SectionHeader title="Terminal" />

            <div className="grid gap-5 xl:grid-cols-2">
              <SettingSection title="Workflow" className="xl:min-h-full">
                <ToggleRow
                  label="Live config reload"
                  note="文件变更后自动让 Alacritty 重新载入"
                  checked={config.merged.general.live_config_reload}
                  onChange={updateLiveReload}
                />
                <SettingRow label="Ownership" note="Import chain">
                  <div className="rounded-2xl border border-[var(--border-strong)] bg-[var(--card)] px-4 py-3 text-sm text-[var(--muted-strong)]">
                    Root / base / theme 分层保持不变。
                  </div>
                </SettingRow>
              </SettingSection>

              <SettingSection title="Cursor" className="xl:min-h-full">
                <SelectRow
                  label="Cursor shape"
                  note="光标形状"
                  value={config.merged.cursor.style.shape}
                  options={Array.from(cursorShapeOptions)}
                  onChange={updateCursorShape}
                />
                <SelectRow
                  label="Blinking"
                  note="光标闪烁策略"
                  value={config.merged.cursor.style.blinking}
                  options={Array.from(cursorBlinkingOptions)}
                  onChange={updateCursorBlinking}
                />
                <RangeRow
                  label="Thickness"
                  note="Underline / Beam 模式粗细"
                  min={0.05}
                  max={0.4}
                  step={0.01}
                  value={config.merged.cursor.thickness}
                  display={config.merged.cursor.thickness.toFixed(2)}
                  onChange={updateCursorThickness}
                />
                <ToggleRow
                  label="Unfocused hollow"
                  note="窗口失焦时使用 hollow cursor"
                  checked={config.merged.cursor.unfocused_hollow}
                  onChange={updateCursorToggle}
                />
                <SelectRow
                  label="Vi mode style"
                  note="Vi 模式是否跟随当前光标"
                  value={config.merged.cursor.vi_mode_style === "None" ? "None" : "FollowCursor"}
                  options={["None", "FollowCursor"]}
                  onChange={(value) => updateViModeStyle(value as "None" | "FollowCursor")}
                />
              </SettingSection>

              <SettingSection title="Selection" className="xl:min-h-full">
                <ToggleRow
                  label="Save to clipboard"
                  note="复制即落系统剪贴板"
                  checked={config.merged.selection.save_to_clipboard}
                  onChange={updateSelectionToggle}
                />
                <TextAreaRow
                  label="Semantic escape chars"
                  note="双击选词时的分隔字符集合"
                  value={config.merged.selection.semantic_escape_chars}
                  onChange={updateSelectionChars}
                />
              </SettingSection>

              <SettingSection title="Scrolling" className="xl:min-h-full">
                <NumberRow
                  label="History"
                  note="滚动历史条数"
                  value={config.merged.scrolling.history}
                  min={0}
                  onChange={(value) => updateScrolling("history", value)}
                />
                <NumberRow
                  label="Multiplier"
                  note="滚轮倍率"
                  value={config.merged.scrolling.multiplier}
                  min={1}
                  onChange={(value) => updateScrolling("multiplier", value)}
                />
              </SettingSection>
            </div>
          </div>
        );
      default:
        return null;
    }
  })();

  return (
    <div data-chrome-theme={chromeTheme} className="app-shell h-full px-5 py-4 text-[var(--foreground)]">
      <div className="mx-auto flex h-full max-w-[1780px] flex-col gap-4">
        <header className="rounded-[24px] border border-[var(--border)] bg-[var(--surface-strong)] px-5 py-4 shadow-[0_18px_48px_rgba(0,0,0,0.24)]">
          <div className="flex flex-wrap items-center justify-between gap-4">
            <div className="min-w-0">
              <div className="text-[11px] font-semibold uppercase tracking-[0.28em] text-[var(--muted-strong)]">
                Alacritty Config UI
              </div>
              <div className="mt-1 truncate text-lg font-semibold text-[var(--foreground)]" title={config?.rootPath}>
                {config ? lastSegment(config.rootPath) : "Loading workspace…"}
              </div>
              <div className="mt-1 text-sm text-[var(--muted-strong)]">Layered root / base / theme ownership</div>
            </div>
            <div className="flex flex-wrap items-center gap-2">
              <ToolbarButton onClick={openConfig} disabled={loading} icon={<FolderOpen className="h-4 w-4" />}>
                Open config
              </ToolbarButton>
              <ToolbarButton onClick={() => void reload()} disabled={loading} icon={<RefreshCw className="h-4 w-4" />}>
                Reload
              </ToolbarButton>
              <ToolbarButton
                onClick={() => void save()}
                disabled={!config || saving || !config.dirty}
                accent
                icon={<Save className="h-4 w-4" />}
              >
                {saving ? "Saving..." : "Save"}
              </ToolbarButton>
            </div>
          </div>

          <div className="mt-3 flex flex-wrap items-center justify-between gap-3 border-t border-[var(--border)] pt-3">
            <nav className="flex flex-wrap items-center gap-2">
              {panels.map((panel) => {
                const Icon = panel.icon;
                return (
                  <button
                    key={panel.id}
                    type="button"
                    onClick={() => selectPanel(panel.id)}
                    className={cn(
                      "inline-flex items-center gap-2 rounded-full border px-4 py-2 text-sm font-medium transition",
                      selectedPanel === panel.id
                        ? "border-[var(--accent)]/35 bg-[var(--surface)] text-[var(--foreground)] shadow-[0_0_0_1px_rgba(134,168,255,0.12)]"
                        : "border-[var(--border-strong)] bg-[var(--card)] text-[var(--muted-strong)] hover:border-[var(--border-strong)] hover:bg-[var(--ghost-hover)] hover:text-[var(--foreground)]",
                    )}
                    title={panel.description}
                  >
                    <Icon className="h-4 w-4" />
                    {panel.label}
                  </button>
                );
              })}
            </nav>
            <div className="flex flex-wrap items-center gap-2 text-xs">
              <div className="inline-flex rounded-full border border-[var(--border-strong)] bg-[var(--card)] p-1">
                {([
                  ["graphite", "Graphite"],
                  ["paper", "Paper"],
                ] as const).map(([value, label]) => (
                  <button
                    key={value}
                    type="button"
                    onClick={() => setChromeTheme(value)}
                    className={cn(
                      "rounded-full px-3 py-1.5 text-xs font-medium transition",
                      chromeTheme === value
                        ? "bg-[var(--accent)] text-[var(--accent-foreground)] shadow-[0_8px_18px_rgba(31,70,199,0.16)]"
                        : "text-[var(--muted-strong)] hover:bg-[var(--ghost-hover)] hover:text-[var(--foreground)]",
                    )}
                  >
                    {label}
                  </button>
                ))}
              </div>
              <span className="rounded-full border border-[var(--border)] bg-[var(--card)] px-3 py-1 font-medium text-[var(--foreground)]/80">
                {isTauriRuntime ? "Desktop runtime" : "Browser preview"}
              </span>
              {config &&
                (config.dirty ? (
                  <span className="rounded-full border border-amber-300/25 bg-amber-300/10 px-3 py-1 font-medium text-amber-100">
                    Dirty · 待保存
                  </span>
                ) : (
                  <span className="inline-flex items-center gap-1 rounded-full border border-[var(--success-border)] bg-[var(--success-bg)] px-3 py-1 font-medium text-[var(--success-fg)]">
                    <CheckCircle2 className="h-3.5 w-3.5" />
                    Synced
                  </span>
                ))}
            </div>
          </div>
        </header>

        {error && (
          <div className="flex items-start gap-3 rounded-2xl border border-rose-400/25 bg-rose-500/10 px-4 py-3 text-sm text-rose-100">
            <AlertCircle className="mt-0.5 h-4 w-4 shrink-0" />
            <div className="min-w-0 flex-1">
              <div className="font-medium">当前链路有异常</div>
              <div className="mt-1 break-words text-rose-100/85">{error}</div>
            </div>
            <button
              type="button"
              onClick={() => setError(null)}
              className="rounded-full px-2 py-1 text-rose-100/70 transition hover:bg-white/10 hover:text-white"
            >
              关闭
            </button>
          </div>
        )}

        <div className="grid min-h-0 flex-1 gap-4 xl:grid-cols-[minmax(0,1fr)_460px]">
          <main className="min-h-0 overflow-y-auto rounded-[24px] border border-[var(--border)] bg-[var(--surface)] px-6 py-6 shadow-[0_18px_48px_rgba(0,0,0,0.24)]">
            {panelContent}
          </main>

          <aside className="min-h-0 space-y-4 overflow-y-auto">
            {config ? <PreviewPane config={config.merged} activeThemePath={config.activeThemePath} /> : <LoadingState compact />}
            {config ? <WorkspacePane config={config} /> : null}
          </aside>
        </div>
      </div>
    </div>
  );
}

function ThemeCard({
  entry,
  active,
  onApply,
}: {
  entry: ThemeEntryDto;
  active: boolean;
  onApply: () => void;
}) {
  const colors = entry.fragment.colors;
  const background = colors?.primary.background ?? "#0F111A";
  const foreground = colors?.primary.foreground ?? "#C5D1EB";
  const accent = colors?.normal.blue ?? "#7AA2F7";
  const cyan = colors?.normal.cyan ?? "#7FDBCA";
  const profile = slugify(entry.name).length;
  const lineWidths = [`${54 + (profile % 14)}%`, `${62 + (profile % 18)}%`, `${34 + (profile % 20)}%`] as const;

  return (
    <button
      type="button"
      onClick={onApply}
      className={cn(
        "group rounded-[22px] border p-3 text-left transition",
        active
          ? "border-[var(--accent)]/35 bg-[var(--accent)]/10 shadow-[0_0_0_1px_rgba(139,167,255,0.12)]"
          : "border-[var(--border)] bg-[var(--card)] hover:border-[var(--border-strong)] hover:bg-[var(--ghost-hover)]",
      )}
    >
      <div className="rounded-[18px] border border-white/8" style={{ backgroundColor: background, color: foreground }}>
        <div className="flex justify-end px-4 py-3 text-[10px] font-semibold uppercase tracking-[0.22em] text-white/70">
          {active ? (
            <span className="rounded-full border border-white/15 bg-white/10 px-2.5 py-1 text-white">
              Current
            </span>
          ) : null}
        </div>
        <div className="border-t border-white/8 px-4 py-4">
          <div className="rounded-[16px] border border-white/10 bg-black/20 px-4 py-3 shadow-[inset_0_1px_0_rgba(255,255,255,0.04)]">
            <div className="flex items-center justify-between text-[10px] font-semibold uppercase tracking-[0.22em] text-white/55">
              <span>Preview</span>
              <span style={{ color: accent }}>●</span>
            </div>
            <div className="mt-4 space-y-2.5">
              <div className="h-2 rounded-full bg-white/8">
                <div className="h-full rounded-full" style={{ width: lineWidths[0], backgroundColor: cyan }} />
              </div>
              <div className="h-2 rounded-full bg-white/8">
                <div
                  className="h-full rounded-full"
                  style={{ width: lineWidths[1], backgroundColor: foreground, opacity: 0.72 }}
                />
              </div>
              <div className="flex items-center gap-2">
                <div className="h-2 w-12 rounded-full" style={{ backgroundColor: accent, opacity: 0.88 }} />
                <div className="h-2 flex-1 rounded-full bg-white/8">
                  <div
                    className="h-full rounded-full"
                    style={{ width: lineWidths[2], backgroundColor: foreground, opacity: 0.42 }}
                  />
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <div className="mt-4 flex items-start justify-between gap-3">
        <div className="min-w-0">
          <div className="truncate text-sm font-semibold text-[var(--foreground)]">{entry.name}</div>
          <div className="mt-1 truncate text-[13px] text-[var(--muted-strong)]" title={entry.path}>
            {lastSegment(entry.path)}
          </div>
        </div>
        <span
          className={cn(
            "shrink-0 rounded-full border px-2.5 py-1 text-[10px] font-semibold uppercase tracking-[0.18em]",
            sourceClasses[entry.source],
          )}
        >
          {sourceLabels[entry.source]}
        </span>
      </div>

      <div className="mt-4 flex items-center justify-between gap-3">
        <div className="flex items-center gap-2">
          {[background, foreground, accent, cyan].map((color) => (
            <ColorDot key={color} color={color} />
          ))}
        </div>
        <span className="inline-flex h-8 items-center rounded-full border border-[var(--border-strong)] bg-[var(--field)] px-3 text-[13px] font-medium text-[var(--muted-strong)] transition group-hover:border-[var(--accent)]/40 group-hover:bg-[var(--accent)]/10 group-hover:text-[var(--foreground)]">Apply</span>
      </div>
    </button>
  );
}

function PreviewPane({
  config,
  activeThemePath,
}: {
  config: EditorConfig;
  activeThemePath: string | null;
}) {
  const background = config.colors.primary.background;
  const foreground = config.colors.primary.foreground;
  const dim = config.colors.primary.dim_foreground ?? "#8A93A8";
  const blue = config.colors.normal.blue;
  const green = config.colors.normal.green;
  const red = config.colors.normal.red;
  const cyan = config.colors.normal.cyan;
  const selectionBg = resolveCellColor(config.colors.selection.background, config.colors);
  const selectionFg = resolveCellColor(config.colors.selection.text, config.colors);
  const cursorBg = resolveCellColor(config.colors.cursor.cursor, config.colors);
  const cursorFg = resolveCellColor(config.colors.cursor.text, config.colors);
  const opacityLabel = `${Math.round(config.window.opacity * 100)}%`;

  return (
    <section className="rounded-[24px] border border-[var(--border)] bg-[var(--surface)] px-5 py-5 shadow-[0_18px_48px_rgba(0,0,0,0.24)]">
      <div className="flex items-center justify-between gap-3">
        <div>
          <div className="text-[11px] font-semibold uppercase tracking-[0.28em] text-[var(--muted-strong)]">
            Live preview
          </div>
          <div className="mt-2 flex items-center gap-2 text-base font-semibold text-[var(--foreground)]">
            <TerminalSquare className="h-4 w-4 text-[var(--accent)]" />
            Terminal canvas
          </div>
        </div>
        <span className="rounded-full border border-[var(--border)] bg-[var(--card)] px-3 py-1 text-xs font-medium text-[var(--foreground)]/80">
          {config.font.normal.family} · {config.font.size.toFixed(2)} pt
        </span>
      </div>

      <div className="mt-4 min-h-[560px] rounded-[22px] border border-white/8 bg-[#090c11] p-4 shadow-[0_24px_60px_rgba(0,0,0,0.3)]">
        <div className="rounded-[18px] border border-white/8 p-3" style={{ backgroundColor: background }}>
          <div className="flex items-center justify-between rounded-[14px] bg-white/5 px-4 py-3">
            <div className="flex items-center gap-2">
              <span className="h-3 w-3 rounded-full bg-[#FF5F56]" />
              <span className="h-3 w-3 rounded-full bg-[#FFBD2E]" />
              <span className="h-3 w-3 rounded-full bg-[#27C93F]" />
            </div>
            <div className="truncate px-3 text-xs uppercase tracking-[0.22em] text-white/55">
              {activeThemePath ? lastSegment(activeThemePath) : "preview.toml"}
            </div>
            <div className="text-xs text-white/45">Opacity {opacityLabel}</div>
          </div>

          <div
            className="min-h-[420px] space-y-4 px-5 py-5"
            style={{
              color: foreground,
              fontFamily: `${config.font.normal.family}, ui-monospace, SFMono-Regular, Menlo, monospace`,
              fontSize: `${config.font.size}px`,
              lineHeight: 1.65,
            }}
          >
            <div className="flex items-center gap-3">
              <span style={{ color: blue }}>~/workspace</span>
              <span style={{ color: dim }}>on</span>
              <span style={{ color: green }}>main</span>
              <span style={{ color: dim }}>•</span>
              <span style={{ color: cyan }}>alacritty-config-ui</span>
            </div>
            <div>
              <span style={{ color: green }}>$</span> <span>cargo tauri dev</span>
            </div>
            <div style={{ color: dim }}>Watching configuration and refreshing preview…</div>
            <div className="rounded-2xl border border-white/8 px-4 py-3" style={{ backgroundColor: selectionBg, color: selectionFg }}>
              selection: semantic boundaries stay aligned with layered imports
            </div>
            <div className="flex items-center gap-3">
              <span style={{ color: red }}>status:</span>
              <span style={{ color: dim }}>preview stays in sync with the current editor state</span>
            </div>
            <div className="flex items-center gap-3 pt-1">
              <span style={{ color: green }}>$</span>
              <CursorSample shape={config.cursor.style.shape} cursorColor={cursorBg} textColor={cursorFg} />
            </div>
          </div>
        </div>
      </div>

    </section>
  );
}

function WorkspacePane({ config }: { config: ConfigStateDto }) {
  return (
    <section className="rounded-[24px] border border-[var(--border)] bg-[var(--surface)] px-5 py-5 shadow-[0_18px_48px_rgba(0,0,0,0.24)]">
      <div>
        <div className="text-[11px] font-semibold uppercase tracking-[0.28em] text-[var(--muted-strong)]">
          Workspace
        </div>
        <div className="mt-2 text-base font-semibold text-[var(--foreground)]">Layered config context</div>
      </div>

      <div className="mt-5 grid gap-3">
        <InfoTile label="Root" value={config.rootPath} />
        <InfoTile label="Base" value={config.basePath ?? "未配置"} />
        <InfoTile label="Theme" value={config.activeThemePath ?? "未配置"} />
      </div>

      <div className="mt-5">
        <div className="text-[11px] font-semibold uppercase tracking-[0.28em] text-[var(--muted-strong)]">
          Runtime summary
        </div>
        <div className="mt-3 grid gap-3 sm:grid-cols-2">
          <InfoTile label="Decorations" value={config.merged.window.decorations} />
          <InfoTile label="Option as Alt" value={config.merged.window.option_as_alt} />
          <InfoTile label="History" value={`${config.merged.scrolling.history} lines`} />
          <InfoTile
            label="Clipboard"
            value={config.merged.selection.save_to_clipboard ? "Auto-save" : "Manual copy"}
          />
        </div>
      </div>

      <div className="mt-5">
        <div className="text-[11px] font-semibold uppercase tracking-[0.28em] text-[var(--muted-strong)]">
          Import chain
        </div>
        <div className="mt-3 space-y-2">
          {config.merged.general.import.map((entry, index) => (
            <div
              key={`${entry}-${index}`}
              className="flex items-center justify-between gap-3 rounded-2xl border border-[var(--border)] bg-[var(--card)] px-4 py-3"
            >
              <span className="text-[11px] font-semibold uppercase tracking-[0.22em] text-[var(--foreground)]">
                {index === 0 ? "Base" : "Theme"}
              </span>
              <span className="truncate text-sm text-[var(--foreground)]" title={entry}>
                {entry}
              </span>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}

function SectionHeader({
  eyebrow,
  title,
  description,
}: {
  eyebrow?: string;
  title: string;
  description?: string;
}) {
  return (
    <div className="space-y-2">
      {eyebrow ? (
        <div className="text-[11px] font-semibold uppercase tracking-[0.28em] text-[var(--muted-strong)]">
          {eyebrow}
        </div>
      ) : null}
      <div>
        <h2 className="text-2xl font-semibold tracking-tight text-[var(--foreground)]">{title}</h2>
        {description ? (
          <p className="mt-2 max-w-3xl text-sm leading-6 text-[var(--muted-strong)]">{description}</p>
        ) : null}
      </div>
    </div>
  );
}

function SettingSection({
  title,
  description,
  children,
  className,
}: {
  title: string;
  description?: string;
  children: React.ReactNode;
  className?: string;
}) {
  return (
    <section className={cn("rounded-[24px] border border-[var(--border)] bg-[var(--card)] px-4 py-4", className)}>
      <div>
        <h3 className="text-lg font-semibold text-[var(--foreground)]">{title}</h3>
        {description ? <p className="mt-1 text-sm leading-6 text-[var(--muted-strong)]">{description}</p> : null}
      </div>
      <div className="mt-4 space-y-2.5">{children}</div>
    </section>
  );
}

function SettingRow({
  label,
  note,
  children,
  alignTop = false,
}: {
  label: string;
  note: string;
  children: React.ReactNode;
  alignTop?: boolean;
}) {
  return (
    <div
      className={cn(
        "grid gap-3 rounded-2xl border border-[var(--border-strong)] bg-[var(--surface)] px-3.5 py-3.5 md:grid-cols-[156px_minmax(0,1fr)]",
        alignTop ? "md:items-start" : "md:items-center",
      )}
    >
      <div className="space-y-1 md:pr-3 md:text-right">
        <div className="text-sm font-semibold text-[var(--foreground)]">{label}</div>
        <div className="text-[13px] leading-5 text-[var(--muted-strong)]">{note}</div>
      </div>
      <div>{children}</div>
    </div>
  );
}

function ToggleRow({
  label,
  note,
  checked,
  onChange,
}: {
  label: string;
  note: string;
  checked: boolean;
  onChange: (checked: boolean) => void;
}) {
  return (
    <SettingRow label={label} note={note}>
      <label className="flex h-10 w-full max-w-[280px] items-center justify-between rounded-2xl border border-[var(--border-strong)] bg-[var(--card)] px-4 text-sm text-[var(--foreground)]">
        <span>{checked ? "Enabled" : "Disabled"}</span>
        <input
          type="checkbox"
          checked={checked}
          onChange={(event) => onChange(event.target.checked)}
          className="h-4 w-4 rounded border-[var(--border-strong)] bg-transparent accent-[var(--accent)]"
        />
      </label>
    </SettingRow>
  );
}

function SelectRow({
  label,
  note,
  value,
  options,
  onChange,
}: {
  label: string;
  note: string;
  value: string;
  options: string[];
  onChange: (value: string) => void;
}) {
  return (
    <SettingRow label={label} note={note}>
      <select
        value={value}
        onChange={(event) => onChange(event.target.value)}
        className="h-10 w-full max-w-[320px] rounded-2xl border border-[var(--border-strong)] bg-[var(--field)] px-4 text-sm text-[var(--foreground)] outline-none transition focus:border-[var(--accent)] focus:bg-[var(--field-focus)]"
      >
        {options.map((option) => (
          <option key={option} value={option}>
            {option}
          </option>
        ))}
      </select>
    </SettingRow>
  );
}

function NumberRow({
  label,
  note,
  value,
  min,
  onChange,
}: {
  label: string;
  note: string;
  value: number;
  min: number;
  onChange: (value: number) => void;
}) {
  return (
    <SettingRow label={label} note={note}>
      <input
        type="number"
        min={min}
        value={value}
        onChange={(event) => onChange(Number(event.target.value))}
        className="h-10 w-full max-w-[180px] rounded-2xl border border-[var(--border-strong)] bg-[var(--field)] px-4 text-sm text-[var(--foreground)] outline-none transition focus:border-[var(--accent)] focus:bg-[var(--field-focus)]"
      />
    </SettingRow>
  );
}

function RangeRow({
  label,
  note,
  min,
  max,
  step,
  value,
  display,
  onChange,
}: {
  label: string;
  note: string;
  min: number;
  max: number;
  step: number;
  value: number;
  display: string;
  onChange: (value: number) => void;
}) {
  return (
    <SettingRow label={label} note={note}>
      <div className="w-full max-w-[360px] rounded-2xl border border-[var(--border-strong)] bg-[var(--card)] px-4 py-3">
        <div className="flex items-center justify-between gap-3">
          <input
            type="range"
            min={min}
            max={max}
            step={step}
            value={value}
            onChange={(event) => onChange(Number(event.target.value))}
            className="w-full accent-[var(--accent)]"
          />
          <span className="shrink-0 rounded-full border border-[var(--border-strong)] bg-[var(--surface-strong)] px-3 py-1 text-xs font-medium text-[var(--foreground)]">
            {display}
          </span>
        </div>
      </div>
    </SettingRow>
  );
}

function TextAreaRow({
  label,
  note,
  value,
  onChange,
}: {
  label: string;
  note: string;
  value: string;
  onChange: (value: string) => void;
}) {
  return (
    <SettingRow label={label} note={note} alignTop>
      <textarea
        value={value}
        onChange={(event) => onChange(event.target.value)}
        className="min-h-24 w-full max-w-[520px] rounded-2xl border border-[var(--border-strong)] bg-[var(--field)] px-4 py-3 font-mono text-sm text-[var(--foreground)] outline-none transition focus:border-[var(--accent)] focus:bg-[var(--field-focus)]"
      />
    </SettingRow>
  );
}

function ColorInputRow({
  label,
  note,
  value,
  onChange,
}: {
  label: string;
  note: string;
  value: HexColor;
  onChange: (value: string) => void;
}) {
  const [draft, setDraft] = useState<string>(value);

  useEffect(() => {
    setDraft(value);
  }, [value]);

  const commit = () => {
    const normalized = normalizeHex(draft);
    if (normalized) {
      setDraft(normalized);
      onChange(normalized);
      return;
    }

    setDraft(value);
  };

  return (
    <SettingRow label={label} note={note}>
      <div className="flex w-full max-w-[280px] items-center gap-2 rounded-2xl border border-[var(--border-strong)] bg-[var(--card)] px-3 py-3">
        <input
          type="color"
          value={value}
          onChange={(event) => {
            const next = event.target.value.toUpperCase();
            setDraft(next);
            onChange(next);
          }}
          className="h-10 w-12 rounded-xl border border-[var(--border-strong)] bg-transparent p-1"
        />
        <input
          value={draft}
          onChange={(event) => setDraft(event.target.value.toUpperCase())}
          onBlur={commit}
          onKeyDown={(event) => {
            if (event.key === "Enter") {
              event.currentTarget.blur();
            }
          }}
          className="h-10 flex-1 rounded-2xl border border-[var(--border-strong)] bg-[var(--field)] px-3 text-sm uppercase tracking-[0.08em] text-[var(--foreground)] outline-none transition focus:border-[var(--accent)] focus:bg-[var(--field-focus)]"
        />
      </div>
    </SettingRow>
  );
}

function MiniToggleCard({
  label,
  checked,
  onChange,
}: {
  label: string;
  checked: boolean;
  onChange: (checked: boolean) => void;
}) {
  return (
    <label className="flex cursor-pointer items-center justify-between rounded-2xl border border-[var(--border-strong)] bg-[var(--card)] px-4 py-2.5 text-sm text-[var(--foreground)]">
      <span>{label}</span>
      <input
        type="checkbox"
        checked={checked}
        onChange={(event) => onChange(event.target.checked)}
        className="h-4 w-4 rounded border-[var(--border-strong)] bg-transparent accent-[var(--accent)]"
      />
    </label>
  );
}

function FontFamilyCard({
  label,
  value,
  options,
  onChange,
}: {
  label: string;
  value: string;
  options: string[];
  onChange: (value: string) => void;
}) {
  return (
    <div className="rounded-2xl border border-[var(--border-strong)] bg-[var(--surface)] px-3.5 py-3.5">
      <div className="text-[11px] font-semibold uppercase tracking-[0.18em] text-[var(--muted-strong)]">
        {label}
      </div>
      <select
        value={value}
        onChange={(event) => onChange(event.target.value)}
        className="mt-3 h-10 w-full rounded-2xl border border-[var(--border-strong)] bg-[var(--field)] px-4 text-sm text-[var(--foreground)] outline-none transition focus:border-[var(--accent)] focus:bg-[var(--field-focus)]"
      >
        {options.map((option) => (
          <option key={option} value={option}>
            {option}
          </option>
        ))}
      </select>
    </div>
  );
}

function CompactNumberField({
  label,
  value,
  min,
  onChange,
}: {
  label: string;
  value: number;
  min: number;
  onChange: (value: number) => void;
}) {
  return (
    <label className="rounded-2xl border border-[var(--border-strong)] bg-[var(--card)] px-3 py-3">
      <div className="text-[11px] font-semibold uppercase tracking-[0.18em] text-[var(--muted-strong)]">
        {label}
      </div>
      <input
        type="number"
        min={min}
        value={value}
        onChange={(event) => onChange(Number(event.target.value))}
        className="mt-2 h-10 w-full rounded-2xl border border-[var(--border-strong)] bg-[var(--field)] px-3 text-sm text-[var(--foreground)] outline-none transition focus:border-[var(--accent)] focus:bg-[var(--field-focus)]"
      />
    </label>
  );
}

function PaletteCard({
  title,
  colors,
}: {
  title: string;
  colors: EditorConfig["colors"]["normal"];
}) {
  return (
    <div className="rounded-2xl border border-[var(--border-strong)] bg-[var(--surface)] px-3.5 py-3.5">
      <div className="text-sm font-semibold text-[var(--foreground)]">{title}</div>
      <div className="mt-3 grid gap-2 sm:grid-cols-2">
        {Object.entries(colors).map(([name, color]) => (
          <div key={name} className="flex items-center gap-2 rounded-xl border border-[var(--border-strong)] bg-[var(--card)] px-2.5 py-2">
            <ColorDot color={color} />
            <div className="min-w-0 text-[11px] font-medium uppercase tracking-[0.12em] text-[var(--muted-strong)]">
              {name}
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}

function InfoTile({ label, value }: { label: string; value: string }) {
  return (
    <div className="rounded-2xl border border-[var(--border-strong)] bg-[var(--surface-strong)] px-4 py-4 shadow-[inset_0_1px_0_rgba(255,255,255,0.03)]">
      <div className="text-[12px] font-semibold uppercase tracking-[0.18em] text-[var(--muted-strong)]">
        {label}
      </div>
      <div className="mt-2 truncate text-[15px] font-medium text-[var(--foreground)]" title={value}>
        {value}
      </div>
    </div>
  );
}

function SummaryPill({ label, value }: { label: string; value: number }) {
  return (
    <div className="inline-flex items-center gap-2 rounded-full border border-[var(--border-strong)] bg-[var(--surface-strong)] px-3 py-2 text-xs shadow-[inset_0_1px_0_rgba(255,255,255,0.03)]">
      <span className="font-medium text-[var(--muted-strong)]">{label}</span>
      <span className="rounded-full bg-[var(--field)] px-2 py-0.5 font-semibold text-[var(--foreground)]">
        {value}
      </span>
    </div>
  );
}

function CursorSample({
  shape,
  cursorColor,
  textColor,
}: {
  shape: CursorShape;
  cursorColor: HexColor;
  textColor: HexColor;
}) {
  if (shape === "Underline") {
    return (
      <span className="inline-flex items-end rounded px-1 text-white">
        <span className="border-b-2 px-1" style={{ borderColor: cursorColor, color: textColor }}>
          a
        </span>
      </span>
    );
  }

  if (shape === "Beam") {
    return (
      <span className="inline-flex items-center gap-1">
        <span className="inline-block h-5 border-l-2" style={{ borderColor: cursorColor }} />
        <span style={{ color: textColor }}>a</span>
      </span>
    );
  }

  return (
    <span className="rounded px-1" style={{ backgroundColor: cursorColor, color: textColor }}>
      a
    </span>
  );
}

function ToolbarButton({
  children,
  onClick,
  disabled,
  accent,
  icon,
}: {
  children: React.ReactNode;
  onClick: () => void;
  disabled?: boolean;
  accent?: boolean;
  icon: React.ReactNode;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      disabled={disabled}
      className={cn(
        "inline-flex h-9 items-center gap-2 rounded-2xl border px-3.5 text-[13px] font-medium transition disabled:cursor-not-allowed disabled:opacity-50",
        accent
          ? "border-[var(--accent)]/30 bg-[var(--accent)] text-[var(--accent-foreground)] hover:brightness-110"
          : "border-[var(--border-strong)] bg-[var(--card)] text-[var(--foreground)] hover:bg-[var(--ghost-hover)]",
      )}
    >
      {icon}
      {children}
    </button>
  );
}

function ColorDot({
  color,
  size = "md",
}: {
  color: string;
  size?: "md" | "lg";
}) {
  return (
    <span
      className={cn(
        "inline-block rounded-full border border-white/15",
        size === "lg" ? "h-10 w-10" : "h-4 w-4",
      )}
      style={{ backgroundColor: color }}
    />
  );
}

function LoadingState({ compact = false }: { compact?: boolean }) {
  return (
    <div
      className={cn(
        "flex h-full items-center justify-center rounded-[24px] border border-dashed border-[var(--border)] bg-[var(--card)] text-center",
        compact ? "min-h-[320px]" : "min-h-[480px]",
      )}
    >
      <div className="space-y-3 px-6">
        <RefreshCw className="mx-auto h-5 w-5 animate-spin text-[var(--accent)]" />
        <div className="text-lg font-semibold text-[var(--foreground)]">正在拉起配置工作台</div>
        <div className="text-sm leading-6 text-[var(--muted-strong)]">
          先加载默认配置，再把主题目录和系统字体一起收回来。
        </div>
      </div>
    </div>
  );
}

function EmptyState({ onRetry }: { onRetry: () => void }) {
  return (
    <div className="flex min-h-[480px] items-center justify-center rounded-[24px] border border-dashed border-[var(--border)] bg-[var(--card)] text-center">
      <div className="space-y-4 px-6">
        <AlertCircle className="mx-auto h-5 w-5 text-amber-300" />
        <div className="text-lg font-semibold text-[var(--foreground)]">还没有拿到可编辑的配置</div>
        <div className="text-sm leading-6 text-[var(--muted-strong)]">
          先把默认配置链路拉起，再继续浏览主题与编辑参数。
        </div>
        <button
          type="button"
          onClick={onRetry}
          className="rounded-2xl border border-[var(--border-strong)] bg-[var(--surface-strong)] px-4 py-2.5 text-sm font-medium text-[var(--foreground)] transition hover:bg-[var(--ghost-hover)]"
        >
          重试载入
        </button>
      </div>
    </div>
  );
}

function isThemeActive(entry: ThemeEntryDto, activePath: string | null) {
  if (!activePath) {
    return false;
  }

  return activePath === entry.path || activePath.toLowerCase().includes(slugify(entry.name));
}

function resolveCellColor(cellColor: CellColor, colors: EditorConfig["colors"]): HexColor {
  if (cellColor === "CellBackground") {
    return colors.primary.background;
  }

  if (cellColor === "CellForeground") {
    return colors.primary.foreground;
  }

  return cellColor;
}

function normalizeHex(value: string): HexColor | null {
  const normalized = value.trim().toUpperCase();
  return /^#[0-9A-F]{6}$/.test(normalized) ? (normalized as HexColor) : null;
}

function toMessage(error: unknown) {
  if (error instanceof Error) {
    return error.message;
  }

  return String(error);
}

function round(value: number, digits: number) {
  const factor = 10 ** digits;
  return Math.round(value * factor) / factor;
}

function lastSegment(value: string) {
  const segments = value.split("/");
  return segments[segments.length - 1] ?? value;
}

function slugify(value: string) {
  return value
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "");
}
