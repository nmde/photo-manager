import type { ValidationResult } from './tags';
import { invoke } from '@tauri-apps/api/core';
import { Photo, type PhotoData } from '@/classes/Photo';

export type Sort = 'name' | 'namedesc' | 'date' | 'datedesc' | 'rating' | 'ratingdesc';

export const initialize = async (path: string) => await invoke<string[]>('initialize', { path });

export async function photo_grid(query: string[], sort: Sort) {
  return Photo.createPhotos(await invoke<PhotoData[]>('photo_grid', { query, sort }));
}

export async function remove_deleted(deleted: string[]) {
  return await invoke('remove_deleted', { deleted });
}

export async function set_photo_title(photo: string, value: string) {
  return await invoke('set_photo_title', { photo, value });
}

export async function set_photo_desc(photo: string, value: string) {
  return await invoke('set_photo_desc', { photo, value });
}

export async function set_photographer(photo: string, value: string) {
  return await invoke('set_photographer', { photo, value });
}

export async function set_photo_people(photo: string, people: string[]) {
  return await invoke('set_photo_people', { photo, value: people });
}

export async function set_photo_location(photo: string, value: string) {
  return await invoke('set_photo_location', { photo, value });
}

export async function set_photo_tags(photo: string, value: string[]) {
  return await invoke<ValidationResult>('set_photo_tags', { photo, value });
}

export async function set_photo_date(photo: string, value: string) {
  return await invoke('set_photo_date', { photo, value });
}

export async function set_photo_group(photo: string, value: string) {
  return await invoke('set_photo_group', { photo, value });
}

export async function set_photo_rating(photo: string, rating: number) {
  return await invoke('set_photo_rating', { photo, rating });
}

export async function set_photo_is_duplicate(photo: string, value: boolean) {
  return await invoke('set_photo_is_duplicate', { photo, value });
}

export async function set_photo_hide_thumbnail(photo: string, value: boolean) {
  return await invoke('set_photo_hide_thumbnail', { photo, value });
}

export const refresh = async (path: string) => await invoke('refresh', { path });
