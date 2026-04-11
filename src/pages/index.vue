<script setup lang="ts">
  import { open } from '@tauri-apps/plugin-dialog';
  import { initialize, remove_deleted } from '@/api/app';
  import { get_setting } from '@/api/settings';
  import { useFileStore } from '@/stores/fileStore';

  const store = useFileStore();
  const router = useRouter();

  const loading = ref(false);
  const interruptDialog = ref(false);
  const deleted = ref<string[]>([]);
  const newPhotos = ref<string[]>([]);
  const initializing = ref(false);
  const deleting = ref(false);
  const initError = ref(false);
  const initErrorMessage = ref('');

  /**
   * Prompts the user to select the folder to manage.
   */
  async function openFolder() {
    loading.value = true;
    const selected = await open({
      directory: true,
      multiple: false,
    });
    if (typeof selected === 'string') {
      initializing.value = true;
      await initialize(selected)
        .ok(d => {
          deleted.value = d.removed;
          newPhotos.value = d.new_photos;
        })
        .err(msg => {
          initError.value = true;
          initErrorMessage.value = msg;
        })
        .send();
      await get_setting('theme')
        .ok(saved_theme => {
          if (saved_theme !== null) {
            store.setTheme(Boolean(saved_theme.value));
          }
        })
        .send();
      if (deleted.value.length > 0 || newPhotos.value.length > 0) {
        interruptDialog.value = true;
      } else {
        await router.push('/tagger');
      }
    }
    loading.value = false;
  }
</script>

<template>
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
  <v-dialog v-model="interruptDialog" persistent>
    <v-card>
      <v-card-text>
        <v-container>
          <v-row>
            <v-col v-if="deleted.length > 0">
              <h2>{{deleted.length}} Missing Files</h2>
              The following files could not be found:
              <ul>
                <li v-for="file in deleted" :key="file">{{ file }}</li>
              </ul>
              <v-btn>Relocate</v-btn>
              <v-btn
                color="primary"
                :loading="deleting"
                @click="
                  async () => {
                    deleting = true;
                    await remove_deleted(deleted);
                    router.push('/tagger');
                  }
                "
              >
                Remove From Project
              </v-btn>
              <v-btn color="primary" :disabled="deleting" @click="router.push('/tagger')">
                Continue Without Removing
              </v-btn>
            </v-col>
            <v-col v-if="newPhotos.length > 0">
              <h2>{{newPhotos.length}} New Files</h2>
              <v-btn color="primary" @click="router.push('/tagger')">Show New Photos</v-btn>
            </v-col>
          </v-row>
        </v-container>
      </v-card-text>
    </v-card>
  </v-dialog>
  <v-dialog v-model="initializing" persistent>
    <v-card title="Initializing">
      <v-card-text>
        <v-progress-linear color="primary" indeterminate />
      </v-card-text>
    </v-card>
  </v-dialog>
  <v-dialog v-model="initError">
    <v-card color="error" title="Could Not Open Folder">
      <v-card-text>
        An error occurred opening the selected folder: {{ initErrorMessage }}
      </v-card-text>
    </v-card>
  </v-dialog>
</template>

<style scoped>
  .main {
    text-align: center;
  }

  .main > h1 {
    margin-bottom: 28px;
  }
</style>
