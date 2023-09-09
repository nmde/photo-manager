<script setup lang="ts">
import { storeToRefs } from 'pinia';
import { useFileStore } from '../stores/fileStore';

const { workingDir } = storeToRefs(useFileStore());
</script>

<template>
  <v-app>
    <v-toolbar>
      <v-btn to="/" icon>
        <v-icon>mdi-home</v-icon>
      </v-btn>
      <v-toolbar-title>{{ workingDir }}</v-toolbar-title>
      <NuxtLink to="/tagger">
        <v-btn>Tagger</v-btn>
      </NuxtLink>
      <NuxtLink to="/collection">
        <v-btn>Map</v-btn>
      </NuxtLink>
      <v-spacer></v-spacer>
    </v-toolbar>
    <slot></slot>
  </v-app>
</template>
