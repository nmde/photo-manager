<script setup lang="ts">
  import type { Photo } from '@/classes/Photo';

  const props = defineProps<{
    photos: Photo[];
    itemsPerRow: number;
    selected: Photo[];
    width: number;
  }>();

  const emit = defineEmits<{
    (e: 'select', photo: Photo, index: number): void;
  }>();

  const scrollerHeight = computed(() => window.innerHeight - 64);

  const size = computed(() => props.width / props.itemsPerRow - 8);

  const photoRows = computed(() => {
    const rows: Photo[][] = [];
    for (let i = 0; i < props.photos.length; i += 1) {
      if (i % props.itemsPerRow === 0) {
        rows.push([]);
      }
      const photo = props.photos[i];
      if (photo) {
        rows.at(-1)?.push(photo);
      }
    }
    return rows;
  });
</script>

<template>
  <div ref="photoScroller">
    <v-virtual-scroll :height="scrollerHeight" :item-height="size" :items="photoRows">
      <template #default="{ item, index }">
        <div class="photo-row">
          <photo-icon
            v-for="(photo, j) in item"
            :key="photo.id"
            :photo="photo"
            :size="size"
            :selected="selected.findIndex(p => p.name === photo.name) >= 0"
            @select="emit('select', photo, index * itemsPerRow + j)"
          />
        </div>
      </template>
    </v-virtual-scroll>
  </div>
</template>

<style scoped>
  .photo-row {
    display: flex;
  }
</style>
