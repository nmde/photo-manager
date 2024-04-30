<script setup lang="ts">
import { ref } from 'vue';
import { Map } from '../../classes/Map';
import { Photo } from '../../classes/Photo';
import { fileStore } from '../../stores/fileStore';

const { filteredPhotos, filters, setFilter, places } = fileStore;

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
  Object.values(places).forEach((place) => {
    map.createMarker(place.pos);
  });
  map.createHeatmap();
  /**
  map.on('markerCreated', async (pos) => {
    selected.value.forEach((photo) => {
      setLocation(photo.data.name, pos);
    });
  });
  map.on('markerClicked', async (pos) => {
    if (selected.value.length > 0) {
      selected.value.forEach((photo) => {
        setLocation(photo.data.name, pos);
      });
    } else {
      filterBy.value = 1;
      setFilter('filterPos', pos);
    }
  });
  */
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

fileStore.on('updatePhoto', () => {
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
                  filterBy = 0;
                  setFilter('enabledTags', tags);
                }
              "
            ></tag-input>
          </div>
          <photo-grid :photos="photos" @select="(s) => (selected = s)"></photo-grid>
        </v-col>
        <v-col cols="6">
          <v-btn :color="selected.length > 0 ? 'primary' : 'default'" flat @click="selected = []"
            >Clear Selection ({{ selected.length }})</v-btn
          >
          <v-checkbox v-model="mapView" label="Show Map"></v-checkbox>
          <div :class="mapView ? '' : 'hidden'">
            <div class="map-container">
              <div ref="mapEl" class="map"></div>
            </div>
            <v-checkbox
              label="Show heatmap"
              v-model="showHeatmap"
              @update:model-value="toggleHeatmap()"
            ></v-checkbox>
          </div>
          <photo-group v-if="selected.length > 0" :photos="selected"></photo-group>
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

.hidden {
  display: none;
}
</style>
