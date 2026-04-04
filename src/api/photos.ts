/**
 * This file provides a TS mirror of src-tauri/src/photos.rs
 */
import { Photo, type PhotoData } from '@/classes/Photo';
import { invoke } from '@tauri-apps/api/core';
import type { ValidationResult } from './tags';

export type Sort = 'name' | 'namedesc' | 'date' | 'datedesc' | 'rating' | 'ratingdesc';

export const initialize = async (path: string) => await invoke<string[]>('initialize', { path });

export const photo_grid = async (query: string[], sort: Sort) =>
  Photo.createPhotos(await invoke<PhotoData[]>('photo_grid', { query, sort }));

export const remove_deleted = async (deleted: string[]) =>
  await invoke('remove_deleted', { deleted });

export const set_photo_title = async (photo: string, value: string) =>
  await invoke('set_photo_title', { photo, value });

export const set_photo_desc = async (photo: string, value: string) =>
  await invoke('set_photo_desc', { photo, value });

export const set_photographer = async (photo: string, value: string) =>
  await invoke('set_photographer', { photo, value });

export const set_photo_people = async (photo: string, people: string[]) =>
  await invoke('set_photo_people', { photo, value: people });

export const set_photo_location = async (photo: string, value: string) =>
  await invoke('set_photo_location', { photo, value });

export const set_photo_tags = async (photo: string, value: string[]) =>
  await invoke<ValidationResult>('set_photo_tags', { photo, value });

export const set_photo_date = async (photo: string, value: string) =>
  await invoke('set_photo_date', { photo, value });

export const set_photo_group = async (photo: string, value: string) =>
  await invoke('set_photo_group', { photo, value });

export const set_photo_rating = async (photo: string, rating: number) =>
  await invoke('set_photo_rating', { photo, rating });

export const set_photo_is_duplicate = async (photo: string, value: boolean) =>
  await invoke('set_photo_is_duplicate', { photo, value });

export const set_photo_hide_thumbnail = async (photo: string, value: boolean) =>
  await invoke('set_photo_hide_thumbnail', { photo, value });

export const refresh = async (path: string) => await invoke('refresh', { path });
