import { invoke } from '@tauri-apps/api/core';
import { APIResult } from '@/classes/APIResult';

export type ThemeSetting = 'Dark' | 'Light';

export function get_theme() {
  return new APIResult<ThemeSetting>(async () => await invoke('get_theme'));
}

export async function set_theme(value: ThemeSetting) {
  await invoke('set_theme', { value });
}

export function get_colors() {
  return new APIResult<string[]>(async () => await invoke('get_colors'));
}

export function promote_color(color: string) {
  return new APIResult<string[]>(async () => await invoke('promote_color', { color }));
}

export function add_color(color: string) {
  return new APIResult<string[]>(async () => await invoke('add_color', { color }));
}
