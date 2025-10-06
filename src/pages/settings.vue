<script setup lang="ts">
import { computed, ref } from 'vue';
import { fileStore } from '../stores/fileStore';

const { encryptJournalEntries, settings, addCamera, cameras, theme, toggleTheme } = fileStore;

const encryptDialog = ref(false);
const password = ref('');
const passwordError = ref('');
const encrypting = ref(false);
const encryptionProgress = ref(0);

const cameraDialog = ref(false);
const cameraName = ref('');
const cameraList = computed(() => Object.values(cameras).toSorted((a, b) => b.count - a.count));

fileStore.on('encryptionProgress', amount => {
  encryptionProgress.value = amount;
});
</script>

<template>
  <v-main class="main">
    <h3>Dark Mode</h3>
    <v-btn @click="toggleTheme()">{{ theme ? 'Dark Mode' : 'Light Mode' }}</v-btn>
    <h3>Cameras</h3>
    <v-btn color="primary" @click="cameraDialog = true">Add Camera</v-btn>
    <div v-for="camera in cameraList" :key="camera.Id">
      {{ camera.data.name }} ({{ camera.count }})
    </div>
    <h3>Encrypt Journal Entries</h3>
    <div v-if="settings.encrypt">Journal entries are encrypted.</div>
    <v-btn
      v-else
      color="primary"
      @click="
        () => {
          encryptDialog = true;
        }
      "
    >
      Start
    </v-btn>
  </v-main>
  <v-dialog v-model="encryptDialog" :persistent="encrypting">
    <v-card>
      <v-card-title>Encrypt Journal Entries</v-card-title>
      <v-card-text>
        <div v-if="encrypting">
          Encrypting journals...
          <v-progress-linear :model-value="encryptionProgress" />
        </div>
        <div v-else>
          Once journal entries are encrypted, you will need to enter your password in the journal
          page to view them.
          <v-text-field
            v-model="password"
            :error-messages="passwordError"
            label="Choose a password"
            type="password"
          />
        </div>
      </v-card-text>
      <v-card-actions>
        <v-btn :disabled="encrypting" @click="encryptDialog = false">Cancel</v-btn>
        <v-btn
          color="primary"
          :loading="encrypting"
          @click="
            async () => {
              if (password.length === 0) {
                passwordError = 'Please enter a password';
              } else {
                passwordError = '';
                encrypting = true;
                await encryptJournalEntries(password);
                encryptDialog = false;
                encrypting = false;
              }
            }
          "
        >
          Start Encryption
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
  <v-dialog v-model="cameraDialog">
    <v-card>
      <v-card-title>Add a Camera</v-card-title>
      <v-card-text>
        <v-text-field v-model="cameraName" label="Camera Name" />
      </v-card-text>
      <v-card-actions>
        <v-btn @click="cameraDialog = false">Cancel</v-btn>
        <v-btn
          color="primary"
          @click="
            async () => {
              await addCamera(cameraName);
              cameraDialog = false;
              cameraName = '';
            }
          "
        >
          Save
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<style>
.main {
  margin: 8px;
}
</style>
