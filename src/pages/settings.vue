<script setup lang="ts">
  import { set_theme } from '@/api/settings';
  import { useFileStore } from '@/stores/fileStore';

  const store = useFileStore();
  const { theme } = storeToRefs(store);

  async function toggleTheme() {
    store.toggleTheme();
    await set_theme(theme.value);
  }
</script>

<template>
  <div class="main">
    <h1>Application Theme</h1>
    <v-btn color="primary" @click="toggleTheme()">
      Switch to {{ theme === 'Dark' ? 'Light' : 'Dark' }} Mode
    </v-btn>
  </div>
</template>

<style>
  .main {
    margin: 18px;
  }
</style>
