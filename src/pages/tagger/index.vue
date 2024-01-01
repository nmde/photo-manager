<script setup lang="ts">
import { storeToRefs } from 'pinia';
import { ref, type VNodeRef } from 'vue';
import { Photo } from '../../classes/Photo';
import { useFileStore } from '../../stores/fileStore';

const fileStore = useFileStore();
const { filteredPhotos, filters } = storeToRefs(fileStore);

const selected = ref<Photo[]>([]);
const gridCol = ref<any>();
const size = ref(0);
const rows = ref(0);

/**
 * Resizes the grid items when the window size changes
 */
function resizeGrid() {
  size.value = gridCol.value?.$el.getBoundingClientRect().width / 4 - 18;
  rows.value = window.innerHeight / size.value;
}

onMounted(() => {
  resizeGrid();
  window.addEventListener('resize', resizeGrid);
});

onUnmounted(() => {
  window.removeEventListener('resize', resizeGrid);
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
              @update="(tags) => (filters.enabledTags = tags)"
            ></tag-input>
          </div>
          <photo-grid
            :photos="filteredPhotos"
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
