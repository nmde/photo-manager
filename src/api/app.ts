import { invoke } from '@tauri-apps/api/core';
import { APIResult } from '@/classes/APIResult';
import { Photo, type PhotoData } from '@/classes/Photo';

export type Sort = 'name' | 'name_desc' | 'date' | 'date_desc' | 'rating' | 'rating_desc';

type LoadedPhotos = {
  removed: string[];
  new_photos: string[];
};

export function initialize(path: string) {
  return new APIResult<LoadedPhotos>(async () => await invoke('initialize', { path }));
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

export async function refresh() {
  await invoke('refresh');
}
