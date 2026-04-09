import type { ValidationResult } from './tags';
import { invoke } from '@tauri-apps/api/core';
import { APIResult } from '@/classes/APIResult';
import { Photo, type PhotoData } from '@/classes/Photo';

export type Sort = 'name' | 'namedesc' | 'date' | 'datedesc' | 'rating' | 'ratingdesc';

export function initialize(path: string) {
  return new APIResult<string[]>(async () => await invoke('initialize', { path }));
}

export function photo_grid(query: string[], sort: Sort) {
  return new APIResult<PhotoData[], Photo[]>(
    async () => await invoke('photo_grid', { query, sort }),
    photos => Photo.createPhotos(photos),
  );
}

export async function remove_deleted(deleted: string[]) {
  await invoke('remove_deleted', { deleted });
}

export async function set_photo_title(photo: string, value: string | null) {
  await invoke('set_photo_title', { photo, value });
}

export async function set_photo_desc(photo: string, value: string | null) {
  await invoke('set_photo_desc', { photo, value });
}

export async function set_photographer(photo: string, value: string | null) {
  await invoke('set_photographer', { photo, value });
}

export async function set_photo_people(photo: string, people: string[]) {
  await invoke('set_photo_people', { photo, value: people });
}

export async function set_photo_location(photo: string, value: string | null) {
  await invoke('set_photo_location', { photo, value });
}

export function set_photo_tags(photo: string, value: string[]) {
  return new APIResult<ValidationResult>(
    async () => await invoke('set_photo_tags', { photo, value }),
  );
}

export async function set_photo_date(photo: string, value: string | null) {
  await invoke('set_photo_date', { photo, value });
}

export async function set_photo_group(photo: string, value: string | null) {
  await invoke('set_photo_group', { photo, value });
}

export async function set_photo_rating(photo: string, rating: number | null) {
  await invoke('set_photo_rating', { photo, rating });
}

export async function set_photo_is_duplicate(photo: string, value: boolean) {
  await invoke('set_photo_is_duplicate', { photo, value });
}

export async function set_photo_hide_thumbnail(photo: string, value: boolean) {
  await invoke('set_photo_hide_thumbnail', { photo, value });
}

export async function refresh(path: string) {
  await invoke('refresh', { path });
}
