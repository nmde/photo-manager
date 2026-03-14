<script setup lang="ts">
  import type { Photo } from '@/classes/Photo';
  import hiddenPng from '../assets/hidden.png';

  const props = defineProps<{
    photo: Photo;
    size: number;
    selected?: boolean;
    hideIcons?: boolean;
  }>();

  const emit = defineEmits<{
    (e: 'select'): void;
  }>();

  const photoPath = computed(() =>
    props.photo.hideThumbnail
      ? hiddenPng
      : (props.photo.video || props.photo.raw
        ? props.photo.thumbnail
        : props.photo.path),
  );

  const displayName = computed(() =>
    props.photo.group === undefined
      ? (props.photo.title.length > 0
        ? props.photo.title
        : props.photo.name)
      : props.photo.group,
  );
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
          <v-icon v-if="props.selected">mdi-check</v-icon>
          <v-icon v-if="!props.photo.valid" color="error">mdi-alert-octagram</v-icon>
          <v-icon v-if="props.photo.hasLocation">mdi-map-marker</v-icon>
          <v-icon v-if="props.photo.date">mdi-calendar</v-icon>
          <v-icon v-if="props.photo.tags.length > 0">mdi-tag-outline</v-icon>
          <v-icon v-if="props.photo.isDuplicate">mdi-content-duplicate</v-icon>
          <v-icon v-if="props.photo.video">mdi-video-outline</v-icon>
          <v-icon v-if="props.photo.group !== undefined">mdi-group</v-icon>
          <v-icon v-if="props.photo.photographer !== undefined">mdi-photo</v-icon>
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

  .photo-name {
    opacity: 0;
  }

  .photo:hover .photo-name {
    opacity: 1;
  }

  .icons {
    background: rgba(0, 0, 0, 0.1);
  }
</style>
