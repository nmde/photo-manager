import { FileEntry } from '@tauri-apps/api/fs';
import { defineStore } from 'pinia';
import { computed, ref } from 'vue';
import { createPhoto, Photo } from '../classes/Photo';

/**
 * Helper method to get lat,lng as a string.
 * @param param0 - The location.
 * @returns The location string.
 */
export function locToString(location?: { lat: number; lng: number }) {
  if (location) {
    return `${location.lat},${location.lng}`;
  }
  return '';
}

/**
 * Helper method to get lat,lng from a string.
 * @param str - The string.
 * @returns The location object.
 */
export function stringToLoc(str: string) {
  const split = str.split(',').map((x) => Number(x));
  return {
    lat: split[0],
    lng: split[1],
  };
}

export const useFileStore = defineStore('files', () => {
  const files = ref<Record<string, Photo>>({});

  const workingDir = ref('');

  const tags = ref<string[]>([]);

  const locations = computed(() => {
    const locRecord: Record<string, string[]> = {};
    Object.values(files.value).forEach((file) => {
      if (file.location !== undefined) {
        const key = locToString(file.location);
        if (!locRecord[key]) {
          locRecord[key] = [];
        }
        locRecord[key].push(file.name);
      }
    });
    return locRecord;
  });

  /**
   * Adds a file to the registry.
   * @param file - The file to add.
   */
  function addFile(file: FileEntry) {
    if (typeof file.name === 'string') {
      files.value[file.name] = createPhoto(file.name, file.path);
    } else {
      throw new Error(`Unexpected file: ${file.path}`);
    }
  }

  /**
   * Sets the working dir name.
   * @param path - The path to the working dir.
   */
  function setWorkingDir(path: string) {
    workingDir.value = path;
  }

  /**
   * Sets the stored photo data for a file.
   * @param name - The name of the file to set.
   * @param data - The data to set.
   */
  function setPhotoData(name: string, data: Photo) {
    files.value[name] = data;
  }

  /**
   * Adds tags.
   * @param newTags - The tags to add.
   */
  function addTags(...newTags: string[]) {
    newTags.forEach((tag) => {
      if (tags.value.indexOf(tag) < 0) {
        tags.value.push(tag);
      }
    });
  }

  return {
    files,
    workingDir,
    tags,
    locations,
    addFile,
    setWorkingDir,
    setPhotoData,
    addTags,
  };
});
