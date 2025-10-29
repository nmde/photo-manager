<script setup lang="ts">
  import type { Photo } from '../classes/Photo';
  import { ref, watch } from 'vue';
  import { fileStore } from '../stores/fileStore';

  const props = defineProps<{
    photos: Photo[];
    prevDate: Date;
  }>();

  const emit = defineEmits<{
    (e: 'update-date', date: Date): void;
  }>();

  const { setDate } = fileStore;

  const current = ref(0);

  const currentPhoto = computed(() => props.photos[current.value]);

  watch(
    () => props.photos,
    () => (current.value = 0),
  );
</script>

<template>
  <div v-if="props.photos.length > 1">
    <v-btn
      flat
      icon
      @click="
        () => {
          if (current > 0) {
            current -= 1;
          }
        }
      "
    >
      <v-icon>mdi-arrow-left</v-icon>
    </v-btn>
    {{ current + 1 }} / {{ props.photos.length }}
    <v-btn
      flat
      icon
      @click="
        () => {
          if (current < props.photos.length - 1) {
            current += 1;
          }
        }
      "
    >
      <v-icon>mdi-arrow-right</v-icon>
    </v-btn>
  </div>
  <photo-detail
    v-if="currentPhoto"
    :photo="currentPhoto"
    :prev-date="prevDate"
    @update:camera="
      async value => {
        for (const photo of props.photos) {
          await photo.setCamera(value);
        }
      }
    "
    @update:date="
      date => {
        props.photos.forEach(photo => {
          setDate(photo.name, date);
        });
        emit('update-date', new Date(date));
      }
    "
    @update:description="
      async description => {
        if (currentPhoto) {
          await currentPhoto.setDescription(description);
        }
      }
    "
    @update:group="
      async group => {
        if (group) {
          for (const photo of props.photos) {
            await photo.setGroup(group);
          }
        }
      }
    "
    @update:hide-thumbnail="
      async value => {
        for (const photo of photos) {
          await photo.setHideThumbnail(value);
        }
      }
    "
    @update:is-duplicate="
      async isDuplicate => {
        if (currentPhoto) {
          await currentPhoto.setDuplicate(isDuplicate);
        }
      }
    "
    @update:location="
      async location => {
        for (const photo of props.photos) {
          await photo.setLocation(location);
        }
      }
    "
    @update:people="
      async people => {
        for (const photo of props.photos) {
          await photo.setPeople(people);
        }
      }
    "
    @update:photographer="
      async value => {
        for (const photo of props.photos) {
          await photo.setPhotographer(value);
        }
      }
    "
    @update:rating="
      async rating => {
        if (currentPhoto) {
          await currentPhoto.setRating(rating);
        }
      }
    "
    @update:tags="
      async tags => {
        console.log(tags);
        for (const photo of props.photos) {
          await photo.setTags(tags);
        }
      }
    "
    @update:title="
      async title => {
        if (currentPhoto) {
          await currentPhoto.setTitle(title);
        }
      }
    "
  />
</template>
