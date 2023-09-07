<script setup lang="ts">
import { storeToRefs } from 'pinia';
import { useFileStore } from './stores/fileStore';

const { workingDir } = storeToRefs(useFileStore());
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
    </v-toolbar>
    <router-view></router-view>
  </v-app>
</template>
