import { invoke } from "@tauri-apps/api/core";
import type { ParsedScript, ScriptMeta, Settings } from "./types";

export const api = {
  listScripts: () => invoke<ScriptMeta[]>("list_scripts"),
  readScript: (id: string) => invoke<string>("read_script", { id }),
  writeScript: (id: string, content: string) =>
    invoke<void>("write_script", { id, content }),
  createScript: (title: string) => invoke<string>("create_script", { title }),
  deleteScript: (id: string) => invoke<void>("delete_script", { id }),
  parseScript: (text: string) => invoke<ParsedScript>("parse_script", { text }),
  getSettings: () => invoke<Settings>("get_settings"),
  saveSettings: (settings: Settings) =>
    invoke<Settings>("save_settings", { settings }),
  openPrompter: (id: string) => invoke<void>("open_prompter", { id }),
  closePrompter: () => invoke<void>("close_prompter"),
  getActiveScript: () => invoke<string | null>("get_active_script"),
};
