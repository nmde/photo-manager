<script setup lang="ts">
import { ref } from 'vue';
import { Photo } from '~/classes/Photo';
import { useFileStore } from '~/stores/fileStore';

const props = defineProps<{
  photos: Photo[];
}>();

const { setTitle, setDescription, updateTags, setRating, setDuplicate, setGroup } = useFileStore();

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
    @update:title="(title) => setTitle(props.photos[current].data.name, title)"
    @update:description="
      (description) => setDescription(props.photos[current].data.name, description)
    "
    @update:tags="
      (tags) => {
        props.photos.forEach((photo) => {
          updateTags(photo.data.name, tags);
        });
      }
    "
    @update:rating="(rating) => setRating(props.photos[current].data.name, rating)"
    @update:is-duplicate="
      (isDuplicate) => setDuplicate(props.photos[current].data.name, isDuplicate)
    "
    @update:group="(group) => setGroup(props.photos[current].data.name, group)"
  ></photo-detail>
</template>
