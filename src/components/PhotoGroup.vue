<script setup lang="ts">
import { ref, watch } from 'vue';
import { Photo } from '../classes/Photo';
import { fileStore } from '../stores/fileStore';
import PhotoDetail from './PhotoDetail.vue';

const props = defineProps<{
  photos: Photo[];
  prevDate: Date;
}>();

const emit = defineEmits<{
  (e: 'updateDate', date: Date): void;
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

watch(
  () => props.photos,
  () => (current.value = 0),
);
</script>

<template>
  <div v-if="props.photos.length > 1">
    <v-btn
      icon
      flat
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
      icon
      flat
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
    :photo="props.photos[current]"
    :prev-date="prevDate"
    @update:title="(title) => setTitle(props.photos[current].data.name, title)"
    @update:description="
      (description) => setDescription(props.photos[current].data.name, description)
    "
    @update:tags="
      (tags) => {
        let expandedTags: string[] = [];
        tags.forEach((tag) => {
          if (tag.indexOf(',') > 0) {
            tag.split(',').forEach((s) => {
              if (expandedTags.indexOf(s) < 0) {
                expandedTags.push(s);
              }
            });
          } else if (expandedTags.indexOf(tag) < 0) {
            expandedTags.push(tag);
          }
        });
        props.photos.forEach((photo) => {
          updateTagsForGroup(photo.data.name, expandedTags);
        });
      }
    "
    @update:rating="(rating) => setRating(props.photos[current].data.name, rating)"
    @update:is-duplicate="
      (isDuplicate) => setDuplicate(props.photos[current].data.name, isDuplicate)
    "
    @update:group="
      (group) => {
        props.photos.forEach((photo) => {
          setGroup(photo.data.name, group);
        });
      }
    "
    @update:date="
      (date) => {
        props.photos.forEach((photo) => {
          setDate(photo.data.name, date);
        });
        emit('updateDate', new Date(date));
      }
    "
    @update:location="
      (location) =>
        props.photos.forEach((photo) => {
          setLocation(photo.data.name, location);
        })
    "
    @update:people="
      (people) => {
        props.photos.forEach((photo) => {
          setPeople(photo.data.name, people);
        });
      }
    "
    @update:photographer="
      (value) => {
        props.photos.forEach((photo) => {
          setPhotographer(photo.data.name, value);
        });
      }
    "
    @update:hide-thumbnail="
      (value) => {
        props.photos.forEach((photo) => {
          setHideThumbnail(photo.data.name, value);
        });
      }
    "
    @update:camera="
      (value) => {
        props.photos.forEach((photo) => {
          setCamera(photo.data.name, value);
        });
      }
    "
  ></photo-detail>
</template>
