import { invoke } from "@tauri-apps/api/core";
import type { CurrentTheme, GlobalThemeSpec } from "../types/theme";

export async function fetchCurrentTheme(): Promise<CurrentTheme> {
  return invoke<CurrentTheme>("current_theme");
}

export async function saveGlobalTheme(
  name: string,
  spec: GlobalThemeSpec
): Promise<string> {
  return invoke<string>("save_global_theme", { name, spec });
}
