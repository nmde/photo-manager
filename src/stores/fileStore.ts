import { FileEntry } from '@tauri-apps/api/fs';
import { defineStore } from 'pinia';
import { computed, ref } from 'vue';
import { locToString } from '../classes/Map';
import { createPhoto, Photo } from '../classes/Photo';

export const useFileStore = defineStore('files', () => {
  const files = ref<Record<string, Photo>>({});

  const workingDir = ref('');

  const tags = ref<string[]>([]);

  const locations = computed(() => {
    const locRecord: Record<string, number> = {};
    Object.values(files.value).forEach((file) => {
      if (file.location !== undefined) {
        const key = locToString(file.location);
        if (!locRecord[key]) {
          locRecord[key] = 0;
        }
        locRecord[key] += 1;
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
   * Sets a photo's thumbnail property.
   * @param photo - The photo to set for.
   * @param thumbnail - The path to the thumbnail.
   */
  function setThumbnail(photo: string, thumbnail: string) {
    files.value[photo].thumbnail = thumbnail;
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

  const photoCount = computed(() => {
    return Object.values(files.value).length;
  });

  /**
   * Sets the photo's location data.
   * @param photo - The target photo.
   * @param location - The location.
   */
  function setLocation(photo: string, location: { lat: number, lng: number }) {
    files.value[photo].location = location;
  }

  /**
   * Moves tags to the front of the list.
   * @param tags - The tags to move to the front.
   */
  function moveTagsToFront(targets: string[]) {
    targets.forEach((tag) => {
      tags.value.splice(tags.value.indexOf(tag), 1);
    });
    tags.value.unshift(...targets);
  }

  return {
    files,
    workingDir,
    tags,
    locations,
    addFile,
    setWorkingDir,
    setPhotoData,
    setThumbnail,
    addTags,
    photoCount,
    setLocation,
    moveTagsToFront,
  };
});
