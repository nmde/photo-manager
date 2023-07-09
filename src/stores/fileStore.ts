import { FileEntry } from '@tauri-apps/api/fs';
import { defineStore } from 'pinia';
import { ref } from 'vue';
import { createPhoto, Photo } from '../classes/Photo';

export const useFileStore = defineStore('files', () => {
  const files = ref<Record<string, Photo>>({});

  const workingDir = ref('');

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

  return {
    files,
    workingDir,
    addFile,
    setWorkingDir,
    setPhotoData,
  };
});
