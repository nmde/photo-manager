<script setup lang="ts">
import { computed, ref } from 'vue';
import { Photo } from '../classes/Photo';
import PhotoIcon from './PhotoIcon.vue';

const props = defineProps<{
  photos: Photo[];
  itemsPerRow: number;
  size: number;
  rows: number;
}>();

const emit = defineEmits<{
  (e: 'select', photo: Photo): void;
}>();

const hideTagged = ref(false);
const hideLocated = ref(false);
const hideDuplicate = ref(false);

type GridRow = Photo[];

// Filters the photos based on the options
const filteredPhotos = computed(() => {
  const rows: GridRow[] = [];
  let row: GridRow = [];
  console.log(props.photos);
  props.photos.forEach((file) => {
    let visible = true;
    if (hideTagged.value === true && file.tags.length > 0) {
      visible = false;
    }
    if (hideLocated.value === true && file.location !== undefined) {
      visible = false;
    }
    if (hideDuplicate.value === true && file.isDuplicate) {
      visible = false;
    }
    if (visible) {
      row.push(file);
      if (row.length === props.itemsPerRow) {
        rows.push(row);
        row = [];
      }
    }
  });
  return rows;
});
</script>

<template>
  <v-toolbar>
    <v-checkbox
      class="collection-control"
      density="compact"
      v-model="hideTagged"
      label="Hide tagged"
    ></v-checkbox>
    <v-checkbox
      class="collection-control"
      density="compact"
      v-model="hideLocated"
      label="Hide located"
    ></v-checkbox>
    <v-checkbox
      class="collection-control"
      density="compact"
      v-model="hideDuplicate"
      label="Hide duplicates"
    ></v-checkbox>
  </v-toolbar>
  <v-virtual-scroll
    :height="props.rows * props.size"
    :item-height="props.size"
    :items="filteredPhotos"
  >
    <template v-slot:default="{ item }">
      <photo-icon
        v-for="(photo, i) in item"
        :key="i"
        :photo="photo"
        :size="props.size"
        @select="emit('select', photo)"
      ></photo-icon>
    </template>
  </v-virtual-scroll>
</template>
