<script setup lang="ts">
import { ref } from 'vue';
import { Photo } from '~/classes/Photo';

const props = defineProps<{
  photos: Photo[];
}>();

const current = ref(0);
</script>

<template>
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
  <photo-detail :photo="props.photos[current]"></photo-detail>
</template>
