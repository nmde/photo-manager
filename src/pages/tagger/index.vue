<script setup lang="ts">
import { ref } from 'vue';
import { Map, type Position } from '../../classes/Map';
import { Photo } from '../../classes/Photo';
import { fileStore } from '../../stores/fileStore';

const { filteredPhotos, filters, setFilter, locations, setLocation } = fileStore;

const selected = ref<Photo[]>([]);
const photos = ref<Photo[]>([]);
const mapView = ref(false);
const mapEl = ref(null);
const showHeatmap = ref(false);
const filterBy = ref(0);

const map = new Map();
onMounted(async () => {
  photos.value = filteredPhotos(filterBy.value);
  await map.initialize(mapEl.value as unknown as HTMLElement);
  Object.entries(locations).forEach(([loc, count]) => {
    map.createMarker(loc, count);
  });
  map.createHeatmap();
  map.on('markerCreated', async (pos) => {
    for (const photo of selected.value) {
      await setLocation(photo.data.name, pos);
    }
  });
  map.on('markerClicked', (pos) => {
    filterBy.value = 1;
    setFilter('filterPos', pos);
  });
});

function toggleHeatmap() {
  if (showHeatmap.value) {
    map.showHeatmap();
    map.hideAllMarkers();
  } else {
    map.hideHeatmap();
    map.showAllMarkers();
  }
}

fileStore.on('updateFilters', () => {
  photos.value = filteredPhotos(filterBy.value);
});

fileStore.on('updatePhoto', (photo) => {
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
              @update="(tags) => {
                filterBy = 0;
                setFilter('enabledTags', tags);
              }"
            ></tag-input>
          </div>
          <photo-grid :photos="photos" @select="(s) => (selected = s)"></photo-grid>
        </v-col>
        <v-col cols="6">
          <v-switch label="Map View" v-model="mapView"></v-switch>
          <photo-group v-if="selected.length > 0 && !mapView" :photos="selected"></photo-group>
          <div>
            <div class="map-container">
              <div ref="mapEl" class="map"></div>
            </div>
            <v-checkbox
              label="Show heatmap"
              v-model="showHeatmap"
              @update:model-value="toggleHeatmap()"
            ></v-checkbox>
          </div>
        </v-col>
      </v-row>
    </v-container>
  </v-main>
</template>

<style scoped>
.map-container {
  flex: 2;
}

.map {
  height: 450px;
}
</style>
