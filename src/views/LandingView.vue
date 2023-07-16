<script setup lang="ts">
import { open } from '@tauri-apps/api/dialog';
import { readDir, exists, readTextFile, createDir, FileEntry } from '@tauri-apps/api/fs';
import { join, appDataDir } from '@tauri-apps/api/path';
import { convertFileSrc } from '@tauri-apps/api/tauri';
import { Command } from '@tauri-apps/api/shell';
import { ref } from 'vue';
import { useRouter } from 'vue-router';
import { Photo } from '../classes/Photo';
import { useFileStore } from '../stores/fileStore';
import { PhotoDataFile } from '../types/photo-data';

const router = useRouter();
const { addFile, setWorkingDir, setPhotoData, addTags, setThumbnail } = useFileStore();

const loading = ref(false);
const deletedDialog = ref(false);
const deleted = ref<string[]>([]);
const thumbnailDialog = ref(false);
const thumbnailCount = ref(0);
const thumbnailProgress = ref(0);

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
    let raws: FileEntry[] = [];
    files.forEach((file) => {
      addFile(file);
      if (/^.*\.(ORF)$/.test(file.path)) {
        raws.push(file);
      }
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
    if (raws.length > 0) {
      thumbnailDialog.value = true;
      thumbnailProgress.value = 0;
      thumbnailCount.value = raws.length;
      const dir = await appDataDir();
      if (!(await exists(dir))) {
        await createDir(dir);
      }
      const thumbnailDir = await join(dir, 'thumbnails');
      if (!(await exists(thumbnailDir))) {
        await createDir(thumbnailDir);
      }
      const projectThumbnailDir = await join(
        thumbnailDir,
        selected.replace(/[/\\]/g, '-').replace(':', ''),
      );
      if (!(await exists(projectThumbnailDir))) {
        await createDir(projectThumbnailDir);
      }
      for (let i = 0; i < raws.length; i += 1) {
        const thumbnailPath = await join(
          projectThumbnailDir,
          `${(raws[i].name as string).replace(/\..*$/, '')}.jpg`,
        );
        if (!(await exists(thumbnailPath))) {
          const convertOutput = await new Command('magick', [
            raws[i].path,
            thumbnailPath,
          ]).execute();
          if (convertOutput.code !== 0) {
            console.error(convertOutput.stderr);
          }
          const resizeOutput = await new Command('magick', [
            thumbnailPath,
            '-resize',
            '800x800',
            thumbnailPath,
          ]).execute();
          if (resizeOutput.code !== 0) {
            console.error(resizeOutput.stderr);
          }
        }
        thumbnailProgress.value += 1;
        setThumbnail(raws[i].name as string, convertFileSrc(thumbnailPath));
      }
      thumbnailDialog.value = false;
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
    <v-dialog v-model="thumbnailDialog" persistent>
      <v-card>
        <v-card-title>Generating Thumbnails</v-card-title>
        <v-card-text>
          Progress: {{ thumbnailProgress }} / {{ thumbnailCount }}
          <v-progress-linear
            :model-value="(thumbnailProgress / thumbnailCount) * 100"
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
