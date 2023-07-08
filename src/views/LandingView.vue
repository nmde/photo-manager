<script setup lang="ts">
import { open } from '@tauri-apps/api/dialog';
import { readDir } from '@tauri-apps/api/fs';
import { ref } from 'vue';
import { useRouter } from 'vue-router';
import { useFileStore } from '../stores/fileStore';

const router = useRouter();
const { addFile, setWorkingDir } = useFileStore();

const loading = ref(false);

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
    (await readDir(selected)).forEach((file) => {
        addFile(file);
    });
    setWorkingDir(selected);
    router.push('/collection');
  }
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
