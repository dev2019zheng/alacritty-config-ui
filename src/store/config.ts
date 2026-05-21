import { create } from "zustand";

import {
  applyThemeEntry,
  listSystemFonts,
  listThemes,
  loadConfig as loadConfigCommand,
  loadDefaultConfig,
  reloadConfig,
  saveConfig,
  updateConfig,
} from "../lib/commands";
import type { ConfigStateDto, EditorConfig, ThemeEntryDto } from "../types/config";

export type PanelId = "theme" | "appearance" | "text" | "terminal";

interface ConfigStore {
  config: ConfigStateDto | null;
  themes: ThemeEntryDto[];
  fonts: string[];
  loading: boolean;
  saving: boolean;
  error: string | null;
  selectedPanel: PanelId;
  bootstrap: () => Promise<void>;
  loadConfig: (path: string) => Promise<void>;
  reload: () => Promise<void>;
  save: () => Promise<void>;
  patchConfig: (mutate: (draft: EditorConfig) => void) => Promise<void>;
  applyTheme: (entry: ThemeEntryDto) => Promise<void>;
  selectPanel: (panel: PanelId) => void;
  setError: (message: string | null) => void;
}

export const useConfigStore = create<ConfigStore>()((set, get) => ({
  config: null,
  themes: [],
  fonts: [],
  loading: false,
  saving: false,
  error: null,
  selectedPanel: "theme",

  async bootstrap() {
    set({ loading: true, error: null });
    try {
      const config = await loadDefaultConfig();
      const [themes, fonts] = await Promise.all([listThemes(), listSystemFonts()]);
      set({ config, themes, fonts, loading: false, error: null });
    } catch (error) {
      set({ loading: false, error: toMessage(error) });
    }
  },

  async loadConfig(path) {
    set({ loading: true, error: null });
    try {
      const config = await loadConfigCommand(path);
      const [themes, fonts] = await Promise.all([listThemes(), listSystemFonts()]);
      set({ config, themes, fonts, loading: false, error: null });
    } catch (error) {
      set({ loading: false, error: toMessage(error) });
    }
  },

  async reload() {
    set({ loading: true, error: null });
    try {
      const config = await reloadConfig();
      const themes = await listThemes();
      set({ config, themes, loading: false, error: null });
    } catch (error) {
      set({ loading: false, error: toMessage(error) });
    }
  },

  async save() {
    set({ saving: true, error: null });
    try {
      const config = await saveConfig();
      set({ config, saving: false, error: null });
    } catch (error) {
      set({ saving: false, error: toMessage(error) });
    }
  },

  async patchConfig(mutate) {
    const current = get().config;
    if (!current) {
      return;
    }

    try {
      const nextMerged = structuredClone(current.merged);
      mutate(nextMerged);
      const config = await updateConfig(nextMerged);
      set({ config, error: null });
    } catch (error) {
      set({ error: toMessage(error) });
    }
  },

  async applyTheme(entry) {
    try {
      const config = await applyThemeEntry(entry);
      const themes = await listThemes();
      set({ config, themes, error: null });
    } catch (error) {
      set({ error: toMessage(error) });
    }
  },

  selectPanel(panel) {
    set({ selectedPanel: panel });
  },

  setError(message) {
    set({ error: message });
  },
}));

function toMessage(error: unknown) {
  if (error instanceof Error) {
    return error.message;
  }

  return String(error);
}
