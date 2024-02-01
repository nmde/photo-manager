<script setup lang="ts">
import { computed } from 'vue';
import { Photo } from '../classes/Photo';

const props = defineProps<{
  photo: Photo;
  size: number;
  selected?: boolean;
  invalid?: boolean;
}>();

const emit = defineEmits<{
  (e: 'select'): void;
}>();

const hasThumbnail = computed(() => {
  return props.photo.data.video || props.photo.data.raw;
});

const photoPath = computed(() => {
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
        class="thumbnail-loading"
        indeterminate
        v-if="hasThumbnail && photo.data.thumbnail.length === 0"
      ></v-progress-circular>
      <v-img
        class="photo-preview align-end text-white"
        aspect-ratio="1/1"
        :width="size"
        :height="size"
        cover
        :src="photoPath"
      >
        <v-card-title class="photo-name">{{ displayName }}</v-card-title>
        <v-icon v-if="props.selected">mdi-check</v-icon>
        <v-icon v-if="props.invalid" color="error">mdi-alert-octagram</v-icon>
        <v-icon v-if="props.photo.location !== undefined">mdi-map-marker</v-icon>
        <v-icon v-if="props.photo.tags.length > 0">mdi-tag-outline</v-icon>
        <v-icon v-if="props.photo.data.isDuplicate">mdi-content-duplicate</v-icon>
        <v-icon v-if="props.photo.data.video">mdi-video-outline</v-icon>
        <v-icon v-if="props.photo.group !== undefined">mdi-group</v-icon>
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
