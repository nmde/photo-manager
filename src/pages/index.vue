<script setup lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { ref } from 'vue';
  import { useRouter } from 'vue-router';
  import { Photo } from '../classes/Photo';
  import { fileStore } from '../stores/fileStore';

  const router = useRouter();
  const { setFiles, removeDeleted } = fileStore;

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
    const { open } = await import('@tauri-apps/plugin-dialog');
    const selected = await open({
      directory: true,
      multiple: false,
    });
    if (selected && typeof selected === 'string') {
      initializing.value = true;
      const data = await invoke<{
        photos: {
          id: string;
          name: string;
          path: string;
          title: string;
          description: string;
          tags: string;
          is_duplicate: number;
          rating: number;
          location: string;
          thumbnail: string;
          video: number;
          photo_group: string;
          date: string;
          raw: number;
          people: string;
          hide_thumbnail: number;
          photographer: string;
          camera: string;
        }[];
        deleted: string[];
      }>('open_folder', { path: selected });
      console.log(deleted);
      const files: Record<string, Photo> = {};
      for (const photo of data.photos.map(
        ({
          id,
          name,
          path,
          title,
          description,
          tags,
          is_duplicate,
          rating,
          location,
          thumbnail,
          video,
          photo_group,
          date,
          raw,
          people,
          hide_thumbnail,
          photographer,
          camera,
        }) =>
          new Photo(
            id,
            name,
            path,
            title,
            description,
            location,
            tags,
            is_duplicate === 1,
            thumbnail,
            rating,
            video === 1,
            photo_group,
            date,
            raw === 1,
            people,
            hide_thumbnail === 1,
            photographer,
            camera,
          ),
      )) {
        files[photo.name] = photo;
      }
      console.log('Loaded photos');
      // const folder = await readDir(selected);
      // setFolderStructure(folder);
      console.log('Read dir');
      deleted.value = data.deleted;
      setFiles(files);
      if (deleted.value.length > 0) {
        deletedDialog.value = true;
      } else {
        await router.push('/tagger');
      }
    }
    loading.value = false;
  }
</script>

<template>
  <v-main>
    <v-container>
      <v-row>
        <v-col cols="4" />
        <v-col cols="12">
          <div class="main">
            <h1>Photo Manager</h1>
            <v-btn color="primary" :loading="loading" @click="openFolder">Open Folder</v-btn>
          </div>
        </v-col>
        <v-col cols="4" />
      </v-row>
    </v-container>
    <v-dialog v-model="deletedDialog" persistent>
      <v-card>
        <v-card-title>Missing Files</v-card-title>
        <v-card-text>
          The following files could not be found:
          <ul>
            <li v-for="(file, i) in deleted" :key="i">{{ file }}</li>
          </ul>
        </v-card-text>
        <v-card-actions>
          <v-btn
            color="primary"
            @click="
              async () => {
                for (let i = 0; i < deleted.length; i += 1) {
                  const d = deleted[i];
                  if (d) {
                    removeDeleted(d);
                  }
                }
                router.push('/tagger');
              }
            "
          >
            Remove Records &amp; Continue
          </v-btn>
          <v-btn color="primary" @click="router.push('/tagger')">Continue Without Removing</v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>
    <v-dialog v-model="initializing" persistent>
      <v-card>
        <v-card-title>Initializing</v-card-title>
        <v-card-text>
          <p v-if="reading.length > 0">Reading {{ reading }}</p>
          <v-progress-linear
            color="primary"
            :model-value="(initializingProgress / fileCount) * 100"
          />
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
