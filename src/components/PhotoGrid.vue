<script setup lang="ts">
import { computed } from 'vue';
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

type GridRow = Photo[];

const rows = computed(() => {
  const rows: GridRow[] = [];
  let row: GridRow = [];
  props.photos.forEach((photo) => {
    row.push(photo);
    if (row.length === props.itemsPerRow) {
      rows.push(row);
      row = [];
    }
  });
  return rows;
});

const visibleRows = computed(() => {
  return props.rows * props.size;
});
</script>

<template>
  <v-virtual-scroll :height="visibleRows" :item-height="props.size" :items="rows">
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
