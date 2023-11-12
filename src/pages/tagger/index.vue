<script setup lang="ts">
import { storeToRefs } from 'pinia';
import { ref } from 'vue';
import { Photo } from '../../classes/Photo';
import { useFileStore } from '../../stores/fileStore';

const fileStore = useFileStore();
const { filteredPhotos, filters } = storeToRefs(fileStore);

const selected = ref<Photo[]>([]);
</script>

<template>
  <v-main class="main">
    <v-container fluid>
      <v-row>
        <v-col cols="6">
          <div class="flex">
            <tag-input
              label="Tags to include"
              :value="filters.enabledTags"
              @update="(tags) => (filters.enabledTags = tags)"
            ></tag-input>
          </div>
          <photo-grid
            :photos="filteredPhotos"
            :items-per-row="4"
            @select="(s) => (selected = s)"
            :size="170"
          ></photo-grid>
        </v-col>
        <v-col cols="6">
          <photo-group v-if="selected.length > 0" :photos="selected"></photo-group>
        </v-col>
      </v-row>
    </v-container>
  </v-main>
</template>

<style scoped></style>
