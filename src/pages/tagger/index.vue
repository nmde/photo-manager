<script setup lang="ts">
import { ref } from 'vue';
import { Photo } from '../../classes/Photo';
import { fileStore } from '../../stores/fileStore';

const route = useRoute();

const { filteredPhotos, filters, setFilter, places } = fileStore;

const selected = ref<Photo[]>([]);
const photos = ref<Photo[]>([]);
const filterBy = ref(0);

fileStore.on('updateFilters', () => {
  photos.value = filteredPhotos(filterBy.value);
});

fileStore.on('updatePhoto', () => {
  photos.value = filteredPhotos(filterBy.value);
});

onMounted(() => {
  if (route.query.place) {
    setFilter('filterPos', route.query.place as string);
    filterBy.value = 1;
  }
  photos.value = filteredPhotos(filterBy.value);
});
</script>

<template>
  <v-main class="main">
    <v-container fluid>
      <v-row>
        <v-col cols="6" ref="gridCol">
          <div class="flex">
            <tag-input
              label="Tags to include"
              :value="filters.enabledTags"
              @update="
                (tags) => {
                  setFilter('enabledTags', tags);
                }
              "
            ></tag-input>
            <div v-if="filterBy === 1">
              <v-btn
                icon
                flat
                @click="
                  () => {
                    filterBy = 0;
                    photos = filteredPhotos(filterBy);
                  }
                "
              >
                <v-icon>mdi-close</v-icon>
              </v-btn>
              {{ places[route.query.place as string].data.name }}
            </div>
          </div>
          <photo-grid :photos="photos" @select="(s) => (selected = s)"></photo-grid>
        </v-col>
        <v-col cols="6">
          <v-btn :color="selected.length > 0 ? 'primary' : 'default'" flat @click="selected = []"
            >Clear Selection ({{ selected.length }})</v-btn
          >
          <photo-group v-if="selected.length > 0" :photos="selected"></photo-group>
        </v-col>
      </v-row>
    </v-container>
  </v-main>
</template>

<style scoped></style>
