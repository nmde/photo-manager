<script setup lang="ts">
import { computed } from 'vue';
import { Photo } from '../classes/Photo';

const props = defineProps<{
  photo: Photo;
  size: number;
  selected?: boolean;
}>();

const emit = defineEmits<{
    (e: 'select'): void;
}>();

const photoPath = computed(() => {
  if (props.photo.thumbnail) {
    return props.photo.thumbnail;
  }
  return props.photo.path;
});
</script>

<template>
  <div
    class="photo"
    :style="{
      height: `${size}px`,
      width: `${size}px`,
    }"
  >
    <v-card @click="emit('select')">
      <v-img
        class="photo-preview align-end text-white"
        aspect-ratio="1/1"
        :width="size"
        :height="size"
        cover
        :src="photoPath"
      >
        <v-card-title class="photo-name">{{ props.photo.name }}</v-card-title>
        <v-icon v-if="props.selected">mdi-check</v-icon>
        <v-icon v-if="props.photo.location !== undefined">mdi-map-marker</v-icon>
        <v-icon v-if="props.photo.tags.length > 0">mdi-tag-outline</v-icon>
        <v-icon v-if="props.photo.isDuplicate">mdi-content-duplicate</v-icon>
      </v-img>
    </v-card>
  </div>
</template>

<style scoped>
.photo,
.photo-name {
  transition: all 100ms ease-in;
}

.photo {
  display: inline-block;
}

.photo-name {
  opacity: 0;
}

.photo:hover .photo-name {
  opacity: 1;
}
</style>
