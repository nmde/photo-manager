<script setup lang="ts">
  import type { Photo } from '@/classes/Photo';
  import hiddenPng from '@/assets/hidden.png';

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
      : props.photo.is_video || props.photo.is_raw
        ? (props.photo.thumbnail as string)
        : props.photo.asset_path,
  );

  const displayName = computed(() =>
    props.photo.group === null
      ? props.photo.title === null
        ? props.photo.name
        : props.photo.title
      : props.photo.group,
  );
</script>

<template>
  <div
    class="photo"
    :class="{ selected: props.selected }"
    :style="{
      height: `${size}px`,
      width: `${size}px`,
    }"
  >
    <v-card rounded="0" @click="emit('select')">
      <v-img
        aspect-ratio="1/1"
        class="photo-preview align-end text-white"
        cover
        :height="size"
        :src="photoPath"
        :width="size"
      >
        <div class="selected-overlay" />
        <v-card-title class="photo-name">{{ displayName }}</v-card-title>
        <div v-if="props.hideIcons !== true" class="icons">
          <div v-if="props.photo.rating !== null">
            <v-icon v-for="i in props.photo.rating" :key="i" color="warning">mdi-star</v-icon>
          </div>
          <v-icon v-if="!props.photo.valid_tags.is_valid" color="error">mdi-alert-octagram</v-icon>
          <v-icon v-if="props.photo.location !== null">mdi-map-marker</v-icon>
          <v-icon v-if="props.photo.date">mdi-calendar</v-icon>
          <v-icon v-if="props.photo.tags.length > 0">mdi-tag-outline</v-icon>
          <v-icon v-if="props.photo.isDuplicate" color="warning">mdi-content-duplicate</v-icon>
          <v-icon v-if="props.photo.is_video">mdi-video-outline</v-icon>
          <v-icon v-if="props.photo.group !== null">mdi-group</v-icon>
          <v-icon v-if="props.photo.photographer !== null">mdi-photo</v-icon>
        </div>
      </v-img>
    </v-card>
  </div>
</template>

<style scoped>
  .photo-name {
    opacity: 0;
    transform: translateY(6px);
    transition:
      opacity var(--duration-fast) var(--ease-out-expo),
      transform var(--duration-fast) var(--ease-out-expo);
  }

  .photo:hover .photo-name {
    opacity: 1;
    transform: translateY(0);
  }

  .photo {
    position: relative;
  }

  .photo::before {
    content: '';
    position: absolute;
    inset: 0;
    border: 2px solid var(--color-primary);
    opacity: 0;
    pointer-events: none;
    z-index: 10;
    transition: opacity var(--duration-fast) var(--ease-out-expo);
  }

  .photo.selected::before {
    opacity: 1;
  }

  .selected-overlay {
    position: absolute;
    inset: 0;
    background: oklch(65% 0.14 245 / 0.1);
    opacity: 0;
    pointer-events: none;
    transition: opacity var(--duration-fast) var(--ease-out-expo);
  }

  .photo.selected .selected-overlay {
    opacity: 1;
  }

  .icons {
    background: oklch(5% 0.003 245 / 0.4);
    padding: var(--space-xs) var(--space-sm);
  }
</style>
