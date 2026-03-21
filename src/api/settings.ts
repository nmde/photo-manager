import type { SettingData } from '@/classes/Setting';
import { invoke } from '@tauri-apps/api/core';

export const set_setting = async (setting: string, value: number) =>
  await invoke('set_setting', { setting, value });

export const get_setting = async (setting: string) =>
  await invoke<SettingData | null>('get_setting', { setting });
