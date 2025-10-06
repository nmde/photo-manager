<script setup lang="ts">
import type { Photo } from '../classes/Photo';
import { computed } from 'vue';
import hiddenPng from '../assets/hidden.png';

const props = defineProps<{
  photo: Photo;
  size: number;
  selected?: boolean;
  invalid?: boolean;
  hideIcons?: boolean;
}>();

const emit = defineEmits<{
  (e: 'select'): void;
}>();

const hasThumbnail = computed(() => props.photo.data.video || props.photo.data.raw);

const photoPath = computed(() => {
  if (props.photo.data.hideThumbnail) {
    return hiddenPng;
  }
  if (hasThumbnail.value) {
    return props.photo.data.thumbnail;
  }
  return props.photo.data.path;
});

const displayName = computed(() => {
  if (props.photo.group !== undefined) {
    return props.photo.group;
  }
  if (props.photo.data.title.length > 0) {
    return props.photo.data.title;
  }
  return props.photo.data.name;
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
      <v-progress-circular
        v-if="hasThumbnail && photo.data.thumbnail.length === 0"
        class="thumbnail-loading"
        indeterminate
      />
      <v-img
        aspect-ratio="1/1"
        class="photo-preview align-end text-white"
        cover
        :height="size"
        :src="photoPath"
        :width="size"
      >
        <v-card-title class="photo-name">{{ displayName }}</v-card-title>
        <div v-if="props.hideIcons !== true" class="icons">
          <div v-if="props.photo.hasRating">
            <v-icon v-for="i in props.photo.rating" :key="i">mdi-star</v-icon>
          </div>
          <div>
            <v-icon v-if="props.selected">mdi-check</v-icon>
            <v-icon v-if="props.invalid" color="error">mdi-alert-octagram</v-icon>
            <v-icon v-if="props.photo.hasLocation">mdi-map-marker</v-icon>
            <v-icon v-if="props.photo.data.date.length > 0">mdi-calendar</v-icon>
            <v-icon v-if="props.photo.tags.length > 0">mdi-tag-outline</v-icon>
            <v-icon v-if="props.photo.data.isDuplicate">mdi-content-duplicate</v-icon>
            <v-icon v-if="props.photo.data.video">mdi-video-outline</v-icon>
            <v-icon v-if="props.photo.group !== undefined">mdi-group</v-icon>
            <v-icon v-if="props.photo.data.photographer !== undefined">mdi-photo</v-icon>
          </div>
        </div>
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

.thumbnail-loading {
  position: absolute;
}
</style>
