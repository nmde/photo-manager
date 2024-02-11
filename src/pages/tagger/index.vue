<script setup lang="ts">
import { ref } from 'vue';
import { Photo } from '../../classes/Photo';
import { fileStore } from '../../stores/fileStore';

const { filteredPhotos, filters, setFilter, files } = fileStore;

const selected = ref<Photo[]>([]);
const gridCol = ref<any>();
const size = ref(0);
const rows = ref(0);
const photos = ref<Photo[]>([]);

/**
 * Resizes the grid items when the window size changes
 */
function resizeGrid() {
  size.value = gridCol.value?.$el.getBoundingClientRect().width / 4 - 18;
  rows.value = window.innerHeight / size.value;
}

onMounted(() => {
  resizeGrid();
  photos.value = filteredPhotos();
  window.addEventListener('resize', resizeGrid);
});

onUnmounted(() => {
  window.removeEventListener('resize', resizeGrid);
});

fileStore.on('updateFilters', () => {
  photos.value = filteredPhotos();
});

fileStore.on('updatePhoto', (photo) => {
  photos.value = filteredPhotos();
});
</script>

<template>
  <v-main class="main">
    <v-container fluid>
      <v-row>
        <v-col cols="6" ref="gridCol">
          <div class="flex">
            <tag-input
              label="Tags to include"
              :value="filters.enabledTags"
              @update="(tags) => setFilter('enabledTags', tags)"
            ></tag-input>
          </div>
          <photo-grid
            :photos="photos"
            :items-per-row="4"
            @select="(s) => (selected = s)"
            :size="size"
            :rows="rows"
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
