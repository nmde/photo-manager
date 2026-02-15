/**
 * This file provides a TS mirror of src-tauri/src/tags.rs
 */
import { Tag, type TagData } from '@/classes/Tag';
import { invoke } from '@tauri-apps/api/core';

export type ValidationResult = {
  is_valid: boolean;
  message: string;
};

export const set_tag_color = async (tag: string, value: string) =>
  await invoke('set_tag_color', { tag, value });

export const set_tag_prereqs = async (tag: string, value: string[]) =>
  await invoke('set_tag_prereqs', { tag, value });

export const set_tag_coreqs = async (tag: string, value: string[]) =>
  await invoke('set_tag_coreqs', { tag, value });

export const set_tag_incompatible = async (tag: string, value: string[]) =>
  await invoke('set_tag_incompatible', { tag, value });

export const get_tags = async () =>
  Tag.createTags(await invoke<Record<string, TagData>>('get_tags'));
