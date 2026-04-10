import { invoke } from '@tauri-apps/api/core';
import { APIResult } from '@/classes/APIResult';
import { Tag, type TagData } from '@/classes/Tag';

export type ValidationResult = {
  is_valid: boolean;
  message: string | null;
};

export async function set_tag_color(tag: string, value: string | null) {
  await invoke('set_tag_color', { tag, value });
}

export async function set_tag_prereqs(tag: string, value: string[]) {
  await invoke('set_tag_prereqs', { tag, value });
}

export async function set_tag_coreqs(tag: string, value: string[]) {
  await invoke('set_tag_coreqs', { tag, value });
}

export async function set_tag_incompatible(tag: string, value: string[]) {
  await invoke('set_tag_incompatible', { tag, value });
}

export function get_tags() {
  return new APIResult<TagData[], Record<string, Tag>>(
    async () => await invoke('get_tags'),
    tags => Tag.createTags(tags),
  );
}

export function validate_photo(photo: string) {
  return new APIResult<ValidationResult>(async () => await invoke('validate_photo', { photo }));
}
