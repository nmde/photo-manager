<script setup lang="ts">
import { storeToRefs } from 'pinia';
import { computed, ref } from 'vue';
import { Photo } from '../../classes/Photo';
import { useFileStore } from '../../stores/fileStore';

const fileStore = useFileStore();
const { getByGroup } = fileStore;
const { files, tags } = storeToRefs(fileStore);

const selected = ref<Photo[]>([]);
const enabledTags = ref<string[]>([]);

const photos = computed(() => {
  const filtered: Photo[] = [];
  Object.values(files.value).forEach((file) => {
    let satisfiesTags = true;
    enabledTags.value.forEach((tag) => {
      if (file.tags.indexOf(tag) < 0) {
        satisfiesTags = false;
      }
    });
    if (satisfiesTags) {
      filtered.push(file);
    }
  });
  return filtered;
});
</script>

<template>
  <v-main class="main">
    <v-container fluid>
      <v-row>
        <v-col cols="6">
          <v-combobox
            label="Tags to include"
            :items="tags"
            multiple
            chips
            clearable
            v-model="enabledTags"
          >
          </v-combobox>
          <photo-grid
            :photos="photos"
            :items-per-row="4"
            @select="(s) => selected = s"
            :size="175"
            :rows="4"
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
