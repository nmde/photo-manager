<script setup lang="ts">
import { storeToRefs } from 'pinia';
import { useFileStore } from '../stores/fileStore';

const { workingDir, saving, saveError, initialized, generatingThumbnails, thumbnailProgress } =
  storeToRefs(useFileStore());
</script>

<template>
  <v-app>
    <v-layout>
      <v-app-bar>
        <v-btn to="/" icon>
          <v-icon>mdi-home</v-icon>
        </v-btn>
        <v-toolbar-title>{{ workingDir }}</v-toolbar-title>
        <v-spacer></v-spacer>
        <span v-if="saving">
          <v-progress-circular indeterminate></v-progress-circular>
        </span>
        <span v-else>Saved</span>
      </v-app-bar>
      <v-navigation-drawer expand-on-hover rail v-if="initialized">
        <v-list density="compact" nav>
          <v-list-item prepend-icon="mdi-image" title="Photos" to="/tagger"></v-list-item>
          <v-list-item prepend-icon="mdi-tag" title="Manage Tags" to="/tags"></v-list-item>
        </v-list>
      </v-navigation-drawer>
      <slot></slot>
    </v-layout>
    <v-snackbar v-model="saveError">Changes could not be saved!</v-snackbar>
    <v-snackbar v-model="generatingThumbnails" :timeout="-1">
      Generating Thumbnails... ({{ thumbnailProgress }}%)
      <v-progress-linear :model-value="thumbnailProgress"></v-progress-linear>
    </v-snackbar>
  </v-app>
</template>
