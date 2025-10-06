<script setup lang="ts">
import { ref, watch } from 'vue';
import { fileStore } from '../stores/fileStore';

const { decryptJournalEntries } = fileStore;

const props = defineProps<{
  value: boolean;
}>();

const password = ref('');
const decrypting = ref(false);
const decryptDialog = ref(false);

watch(props, () => {
  decryptDialog.value = props.value;
});
</script>

<template>
  <v-dialog v-model="decryptDialog" :persistent="decrypting">
    <v-card>
      <v-card-title>Decrypt Journal & Wiki</v-card-title>
      <v-card-text>
        <v-text-field v-model="password" label="Enter password" type="password" />
      </v-card-text>
      <v-card-actions>
        <v-btn :disabled="decrypting" @click="decryptDialog = false">Cancel</v-btn>
        <v-btn
          color="primary"
          :loading="decrypting"
          @click="
            async () => {
              decrypting = true;
              decryptJournalEntries(password);
              decryptDialog = false;
              decrypting = false;
            }
          "
        >
          Decrypt
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>
