import type { ValidationResult } from './tags';
import type { PhotoData } from '@/classes/Photo';
import { invoke } from '@tauri-apps/api/core';
import { APIResult } from '@/classes/APIResult';

export async function set_photo_title(photo: PhotoData['name'], value: PhotoData['title']) {
  await invoke('set_photo_title', { photo, value });
}

export async function set_photo_desc(photo: PhotoData['name'], value: PhotoData['description']) {
  await invoke('set_photo_desc', { photo, value });
}

export async function set_photographer(photo: PhotoData['name'], value: PhotoData['photographer']) {
  await invoke('set_photographer', { photo, value });
}

export async function set_photo_people(photo: PhotoData['name'], people: PhotoData['people']) {
  await invoke('set_photo_people', { photo, value: people });
}

export async function set_photo_location(photo: PhotoData['name'], value: PhotoData['location']) {
  await invoke('set_photo_location', { photo, value });
}

export function set_photo_tags(photo: PhotoData['name'], value: PhotoData['tags']) {
  return new APIResult<ValidationResult>(
    async () => await invoke('set_photo_tags', { photo, value }),
  );
}

export async function set_photo_date(photo: PhotoData['name'], value: PhotoData['date']) {
  await invoke('set_photo_date', { photo, value });
}

export async function set_photo_group(photo: PhotoData['name'], value: PhotoData['photo_group']) {
  await invoke('set_photo_group', { photo, value });
}

export async function set_photo_rating(photo: PhotoData['name'], rating: PhotoData['rating']) {
  await invoke('set_photo_rating', { photo, rating });
}

export async function set_photo_is_duplicate(
  photo: PhotoData['name'],
  value: PhotoData['is_duplicate'],
) {
  await invoke('set_photo_is_duplicate', { photo, value });
}

export async function set_photo_hide_thumbnail(
  photo: PhotoData['name'],
  value: PhotoData['hide_thumbnail'],
) {
  await invoke('set_photo_hide_thumbnail', { photo, value });
}

export async function get_grouped_raw(photo: PhotoData['name']) {
  return await invoke<string | null>('get_grouped_raw', { photo });
}
