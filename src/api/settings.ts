import type { SettingData } from '@/classes/Setting';
import type { Nullable } from '@/types';
import { invoke } from '@tauri-apps/api/core';
import { APIResult } from '@/classes/APIResult';

export async function set_setting(setting: string, value: number) {
  await invoke('set_setting', { setting, value });
}

export function get_setting(setting: string) {
  return new APIResult<Nullable<SettingData>>(async () => await invoke('get_setting', { setting }));
}
