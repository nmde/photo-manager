import type { Nullable } from '@/types';
import { invoke } from '@tauri-apps/api/core';
import { APIResult } from '@/classes/APIResult';
import { Tag, type TagData, type TagRec } from '@/classes/Tag';

export type ValidationResult = {
  is_valid: boolean;
  message: Nullable<string>;
};

export async function set_tag_color(tag: TagData['name'], value: TagData['color']) {
  await invoke('set_tag_color', { tag, value });
}

export async function set_tag_prereqs(tag: TagData['name'], value: TagData['prereqs']) {
  await invoke('set_tag_prereqs', { tag, value });
}

export async function set_tag_coreqs(tag: TagData['name'], value: TagData['coreqs']) {
  await invoke('set_tag_coreqs', { tag, value });
}

export async function set_tag_incompatible(tag: TagData['name'], value: TagData['incompatible']) {
  await invoke('set_tag_incompatible', { tag, value });
}

export function get_tags() {
  return new APIResult<TagData[], TagRec>(
    async () => await invoke('get_tags'),
    tags => Tag.createTags(tags),
  );
}

export function validate_photo(photo: string) {
  return new APIResult<ValidationResult>(async () => await invoke('validate_photo', { photo }));
}
