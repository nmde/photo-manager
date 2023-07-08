import { FileEntry } from '@tauri-apps/api/fs';
import { defineStore } from 'pinia';
import { ref } from 'vue';
import { Photo } from '../classes/Photo';

export const useFileStore = defineStore('files', () => {
  const files = ref<Record<string, Photo>>({});

  const workingDir = ref('');

  /**
   * Adds a file to the registry.
   * @param file - The file to add.
   */
  function addFile(file: FileEntry) {
    if (typeof file.name === 'string') {
      files.value[file.name] = new Photo(file.name, file.path);
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

  return {
    files,
    workingDir,
    addFile,
    setWorkingDir,
  };
});
