<script setup lang="ts">
import { open } from '@tauri-apps/api/dialog';
import { readDir, exists, readTextFile } from '@tauri-apps/api/fs';
import { join } from '@tauri-apps/api/path';
import { ref } from 'vue';
import { useRouter } from 'vue-router';
import { Photo } from '../classes/Photo';
import { useFileStore } from '../stores/fileStore';
import { PhotoDataFile } from '../types/photo-data';

const router = useRouter();
const { addFile, setWorkingDir, setPhotoData, addTags } = useFileStore();

const loading = ref(false);
const deletedDialog = ref(false);
const deleted = ref<string[]>([]);

/**
 * Prompts the user to select the folder to manage.
 */
async function openFolder() {
  loading.value = true;
  const selected = await open({
    directory: true,
    multiple: false,
  });
  if (selected && typeof selected === 'string') {
    const files = await readDir(selected);
    files.forEach((file) => {
      addFile(file);
    });
    setWorkingDir(selected);
    const photoManagerFile = await join(selected, 'photo-data.json');
    if (await exists(photoManagerFile)) {
      const photoData = JSON.parse(await readTextFile(photoManagerFile)) as PhotoDataFile;
      Object.entries(photoData.files).forEach(([name, data]) => {
        if (!files.find((f) => f.name === name)) {
          deleted.value.push(name);
        } else {
          setPhotoData(name, data as Photo);
        }
      });
      addTags(...photoData.tags);
    }
    if (deleted.value.length > 0) {
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
          <v-card>
            <v-card-text class="main">
              <h1>Photo Manager</h1>
              <v-btn color="primary" @click="openFolder" :loading="loading">Open Folder</v-btn>
            </v-card-text>
          </v-card>
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
