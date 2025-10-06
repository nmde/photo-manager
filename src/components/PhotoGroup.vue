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

const {
  setTitle,
  setDescription,
  updateTagsForGroup,
  setRating,
  setDuplicate,
  setGroup,
  setDate,
  setLocation,
  setPeople,
  setHideThumbnail,
  setPhotographer,
  setCamera,
} = fileStore;

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
      value => {
        props.photos.forEach(photo => {
          setCamera(photo.data.name, value);
        });
      }
    "
    @update:date="
      date => {
        props.photos.forEach(photo => {
          setDate(photo.data.name, date);
        });
        emit('update-date', new Date(date));
      }
    "
    @update:description="
      description => {
        if (currentPhoto) {
          setDescription(currentPhoto.data.name, description);
        }
      }
    "
    @update:group="
      group => {
        props.photos.forEach(photo => {
          setGroup(photo.data.name, group);
        });
      }
    "
    @update:hide-thumbnail="
      value => {
        props.photos.forEach(photo => {
          setHideThumbnail(photo.data.name, value);
        });
      }
    "
    @update:is-duplicate="
      isDuplicate => {
        if (currentPhoto) {
          setDuplicate(currentPhoto.data.name, isDuplicate);
        }
      }
    "
    @update:location="
      location =>
        props.photos.forEach(photo => {
          setLocation(photo.data.name, location);
        })
    "
    @update:people="
      people => {
        props.photos.forEach(photo => {
          setPeople(photo.data.name, people);
        });
      }
    "
    @update:photographer="
      value => {
        props.photos.forEach(photo => {
          setPhotographer(photo.data.name, value);
        });
      }
    "
    @update:rating="
      rating => {
        if (currentPhoto) {
          setRating(currentPhoto.data.name, rating);
        }
      }
    "
    @update:tags="
      (tags) => {
        let expandedTags: string[] = [];
        tags.forEach((tag) => {
          if (tag.indexOf(',') > 0) {
            tag.split(',').forEach((s) => {
              if (!expandedTags.includes(s)) {
                expandedTags.push(s);
              }
            });
          } else if (!expandedTags.includes(tag)) {
            expandedTags.push(tag);
          }
        });
        props.photos.forEach((photo) => {
          updateTagsForGroup(photo.data.name, expandedTags);
        });
      }
    "
    @update:title="
      title => {
        if (currentPhoto) {
          setTitle(currentPhoto.data.name, title);
        }
      }
    "
  />
</template>
