import type { SettingData } from '@/classes/Setting';
import { invoke } from '@tauri-apps/api/core';

export async function set_setting(setting: string, value: number) {
  return await invoke('set_setting', { setting, value });
}

export async function get_setting(setting: string) {
  return await invoke<SettingData | null>('get_setting', { setting });
}
