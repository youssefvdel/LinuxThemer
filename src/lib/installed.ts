import { invoke } from "@tauri-apps/api/core";
import type { InstalledTheme } from "../types/theme";

export async function fetchInstalled(): Promise<InstalledTheme[]> {
  return invoke<InstalledTheme[]>("list_installed");
}

export async function saveCurrentTheme(name: string): Promise<string> {
  return invoke<string>("save_current_theme", { name });
}

export async function launchStudio(kind: string): Promise<string> {
  return invoke<string>("launch_studio", { kind });
}

export async function removeInstalled(path: string): Promise<void> {
  return invoke<void>("remove_installed", { path });
}
