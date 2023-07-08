<script setup lang="ts">
import { Photo } from '../classes/Photo';

const props = defineProps<{
  photo: Photo;
  size: number;
}>();

const emit = defineEmits<{
    (e: 'select'): void;
}>();
</script>

<template>
  <div
    class="photo"
    :style="{
      height: `${size}px`,
      width: `${size}px`,
      maxHeight: `${size}px`,
      maxWidth: `${size}px`,
    }"
  >
    <v-card @click="emit('select')">
      <v-img
        class="photo-preview align-end text-white"
        aspect-ratio="1/1"
        :width="size"
        :height="size"
        cover
        :src="props.photo.path"
      >
        <v-card-title class="photo-name">{{ props.photo.name }}</v-card-title>
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
