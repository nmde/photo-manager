<script setup lang="ts">
  import type { Photo } from '@/classes/Photo';

  const props = defineProps<{
    photos: Photo[];
    itemsPerRow: number;
    selected: Photo[];
    halfWidth: boolean;
  }>();

  const emit = defineEmits<{
    (e: 'select', photo: Photo, index: number): void;
  }>();

  const scrollerHeight = ref(0);
  const scrollerWidth = ref(0);
  const adjustedWidth = computed(() => scrollerWidth.value / (props.halfWidth ? 2 : 1));
  const size = computed(() => adjustedWidth.value / props.itemsPerRow - 8);

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

  function resize() {
    scrollerHeight.value = window.innerHeight - 64;
    scrollerWidth.value = window.innerWidth - (props.halfWidth ? 22 : 40);
  }

  onMounted(() => {
    resize();
    window.addEventListener('resize', () => {
      resize();
    });
  });
</script>

<template>
  <div ref="photoScroller">
    <v-virtual-scroll
      :height="scrollerHeight"
      :item-height="size"
      :items="photoRows"
      :width="adjustedWidth"
    >
      <template #default="{ item, index }">
        <div class="photo-row">
          <photo-icon
            v-for="(photo, j) in item"
            :key="photo.name"
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
