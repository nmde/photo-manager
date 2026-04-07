import { invoke } from '@tauri-apps/api/core';
import { Tag, type TagData } from '@/classes/Tag';

export type ValidationResult = {
  is_valid: boolean;
  message: string;
};

export async function set_tag_color(tag: string, value: string) {
  return await invoke('set_tag_color', { tag, value });
}

export async function set_tag_prereqs(tag: string, value: string[]) {
  return await invoke('set_tag_prereqs', { tag, value });
}

export async function set_tag_coreqs(tag: string, value: string[]) {
  return await invoke('set_tag_coreqs', { tag, value });
}

export async function set_tag_incompatible(tag: string, value: string[]) {
  return await invoke('set_tag_incompatible', { tag, value });
}

export async function get_tags() {
  return Tag.createTags(await invoke<TagData[]>('get_tags'));
}

export async function validate_photo(photo: string) {
  return await invoke<ValidationResult>('validate_photo', { photo });
}
