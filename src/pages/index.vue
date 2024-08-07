<script setup lang="ts">
import { ref } from 'vue';
import { useRouter } from 'vue-router';
import { fileStore } from '../stores/fileStore';
import { Photo, createPhoto } from '~/classes/Photo';
import type { FileEntry } from '@tauri-apps/api/fs';

const router = useRouter();
const { setWorkingDir, loadPhotos, setFiles, generateThumbnails, groupRaws, removeDeleted } =
  fileStore;

const loading = ref(false);
const deletedDialog = ref(false);
const deleted = ref<string[]>([]);
const initializing = ref(false);
const initializingProgress = ref(0);
const fileCount = ref(0);
const reading = ref('');

/**
 * Prompts the user to select the folder to manage.
 */
async function openFolder() {
  loading.value = true;
  const { open } = await import('@tauri-apps/api/dialog');
  const { readDir } = await import('@tauri-apps/api/fs');
  const { convertFileSrc } = await import('@tauri-apps/api/tauri');
  const selected = await open({
    directory: true,
    multiple: false,
  });
  if (selected && typeof selected === 'string') {
    initializing.value = true;
    await setWorkingDir(selected);
    const files: Record<string, Photo> = {};
    const existing = { ...(await loadPhotos()) };
    const fullFileList: any[] = [];
    let raws: any[] = [];
    let videos: any[] = [];
    const expandDir = async (entries: FileEntry[]) => {
      for (const file of entries) {
        if (file.children !== undefined) {
          reading.value = file.path;
          console.log(`Reading ${file.path}`);
          initializingProgress.value += 1;
          await expandDir(await readDir(file.path));
        } else {
          fullFileList.push(file);
        }
      }
    };
    console.log('Loaded photos');
    const dir = await readDir(selected);
    console.log('Read dir');
    fileCount.value = dir.length;
    await expandDir(dir);
    const rawPhotos: Photo[] = [];
    fullFileList.forEach(async (file) => {
      if (existing[file.path]) {
        files[file.path] = existing[file.path];
        delete existing[file.path];
      } else {
        files[file.path] = createPhoto(file.path, convertFileSrc(file.path));
      }
      if (/^.*\.(ORF|NRW|HEIC|TIFF)$/.test(file.path.toUpperCase())) {
        files[file.path].data.raw = true;
        raws.push(file);
      } else if (/^.*\.(3GP|AVI|MOV|MP4|MTS|WAV|WMV|M4V|WEBM)$/.test(file.path.toUpperCase())) {
        files[file.path].data.video = true;
        videos.push(file);
      }
      if (files[file.path].data.raw) {
        rawPhotos.push(files[file.path]);
      }
    });
    deleted.value = Object.keys(existing);
    for (let i = 0; i < deleted.value.length; i += 1) {
      await removeDeleted(deleted.value[i]);
    }
    setFiles(files);
    groupRaws(rawPhotos);
    if (raws.length > 0 || videos.length > 0) {
      generateThumbnails(raws, videos);
      if (deleted.value.length > 0) {
        deletedDialog.value = true;
      } else {
        router.push('/tagger');
      }
    } else if (deleted.value.length > 0) {
      deletedDialog.value = true;
    } else {
      router.push('/tagger');
    }
  }
  loading.value = false;
}
</script>

<template>
  <v-main>
    <v-container>
      <v-row>
        <v-col cols="4"></v-col>
        <v-col cols="12">
          <div class="main">
            <h1>Photo Manager</h1>
            <v-btn color="primary" @click="openFolder" :loading="loading">Open Folder</v-btn>
          </div>
        </v-col>
        <v-col cols="4"></v-col>
      </v-row>
    </v-container>
    <v-dialog v-model="deletedDialog">
      <v-card>
        <v-card-title>Missing Files</v-card-title>
        <v-card-text>
          The following files could not be found:
          <ul>
            <li v-for="(file, i) in deleted" :key="i">{{ file }}</li>
          </ul>
        </v-card-text>
        <v-card-actions>
          <v-btn color="primary" @click="router.push('/tagger')">Continue</v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>
    <v-dialog v-model="initializing" persistent>
      <v-card>
        <v-card-title>Initializing</v-card-title>
        <v-card-text>
          <p v-if="reading.length > 0">Reading {{ reading }}</p>
          <v-progress-linear
            :model-value="(initializingProgress / fileCount) * 100"
            color="primary"
          ></v-progress-linear>
        </v-card-text>
      </v-card>
    </v-dialog>
  </v-main>
</template>

<style scoped>
.main {
  text-align: center;
}

.main > h1 {
  margin-bottom: 28px;
}
</style>
