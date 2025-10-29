<script setup lang="ts">
  import { ref } from 'vue';
  import { useRouter } from 'vue-router';
  import { fileStore } from '../stores/fileStore';

  const router = useRouter();
  const { removeDeleted, loadPhotos } = fileStore;

  const loading = ref(false);
  const deletedDialog = ref(false);
  const deleted = ref<string[]>([]);
  const initializing = ref(false);
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
      await loadPhotos(selected);
      console.log('Loaded photos');
      // const folder = await readDir(selected);
      // setFolderStructure(folder);
      /*
      deleted.value = data.deleted;
      setFiles(files);
      if (deleted.value.length > 0) {
        deletedDialog.value = true;
      } else {
        await router.push('/tagger');
      }
      */
      await router.push('/tagger');
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
            indeterminate
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
