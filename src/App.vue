<script setup lang="ts">
import { writeTextFile } from '@tauri-apps/api/fs';
import { join } from '@tauri-apps/api/path';
import { storeToRefs } from 'pinia';
import { ref } from 'vue';
import { useFileStore } from './stores/fileStore';
import { PhotoDataFile } from './types/photo-data';

const { workingDir, files, tags, groups } = storeToRefs(useFileStore());

const saving = ref(false);
const saved = ref(false);

/**
 * Saves the photo data.
 */
async function save() {
  saving.value = true;
  const photoManagerFile = await join(workingDir.value, 'photo-data.json');
  const photoData: PhotoDataFile = {
    files: files.value,
    locations: {},
    groups: groups.value,
  };
  await writeTextFile(photoManagerFile, JSON.stringify(photoData));
  saving.value = false;
  saved.value = true;
}
</script>

<template>
  <v-app>
    <v-toolbar>
      <v-btn to="/" icon>
        <v-icon>mdi-home</v-icon>
      </v-btn>
      <v-toolbar-title>{{ workingDir }}</v-toolbar-title>
      <v-btn to="/tagger">Tagger</v-btn>
      <v-btn to="/collection">Collection</v-btn>
      <v-spacer></v-spacer>
      <v-btn color="primary" @click="save" :loading="saving">Save</v-btn>
    </v-toolbar>
    <router-view></router-view>
    <v-snackbar v-model="saved">Collection saved</v-snackbar>
  </v-app>
</template>
