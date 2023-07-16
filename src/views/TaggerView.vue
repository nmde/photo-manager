<script setup lang="ts">
import { Loader } from '@googlemaps/js-api-loader';
import { storeToRefs } from 'pinia';
import { computed, ref } from 'vue';
import { Photo, createPhoto } from '../classes/Photo';
import PhotoGrid from '../components/PhotoGrid.vue';
import { useFileStore, stringToLoc, locToString } from '../stores/fileStore';

import { onMounted } from 'vue';

const fileStore = useFileStore();
const { addTags } = fileStore;
const { files, tags, locations } = storeToRefs(fileStore);

const selected = ref<Photo>(createPhoto('', ''));
const hasSelected = ref(false);
const mapEl = ref(null);
const photoView = ref(false);

const photos = computed(() => {
  return Object.values(files.value);
});

let map: google.maps.Map;
let placedMarker = false;
let GoogleAdvancedMarkerElement: typeof google.maps.marker.AdvancedMarkerElement;
const markers: Record<string, google.maps.marker.AdvancedMarkerElement> = {};

/**
 * Creates a marker on the map.
 * @param pos - The position to create the marker at.
 */
function createMarker(pos: string) {
  if (!markers[pos]) {
    markers[pos] = new GoogleAdvancedMarkerElement({
      map: map,
      position: stringToLoc(pos),
      title: selected.value?.name,
      gmpDraggable: true,
    });
    google.maps.event.addListener(markers[pos], 'click', () => {
      selected.value.location = stringToLoc(pos);
    });
  }
}

/**
 * Select a photo to edit.
 * @param photo - The photo.
 */
function selectPhoto(photo: Photo) {
  selected.value = photo;
  hasSelected.value = true;
  if (selected.value.location !== undefined) {
    placedMarker = true;
    map.setCenter(selected.value.location);
  } else {
    placedMarker = false;
  }
}

onMounted(() => {
  new Loader({
    apiKey: import.meta.env.VITE_GOOGLE_MAPS_KEY,
    version: 'weekly',
  })
    .load()
    .then(async () => {
      const { Map } = (await google.maps.importLibrary('maps')) as google.maps.MapsLibrary;
      const { AdvancedMarkerElement } = (await google.maps.importLibrary(
        'marker',
      )) as google.maps.MarkerLibrary;
      GoogleAdvancedMarkerElement = AdvancedMarkerElement;

      map = new Map(mapEl.value as unknown as HTMLElement, {
        zoom: 6,
        mapId: 'DEMO_MAP_ID',
      });

      navigator.geolocation.getCurrentPosition((position: GeolocationPosition) => {
        map.setCenter({
          lat: position.coords.latitude,
          lng: position.coords.longitude,
        });
      });

      map.addListener('dblclick', (e: google.maps.MapMouseEvent) => {
        if (!placedMarker && hasSelected.value) {
          createMarker(locToString(e.latLng?.toJSON()));
          placedMarker = true;
          selected.value.location = e.latLng?.toJSON();
        }
      });

      Object.entries(locations.value).forEach(([loc]) => {
        createMarker(loc);
      });
    });
});

/**
 * Adds new tags to the master list.
 */
function updateTags() {
  selected.value.tags.forEach((tag) => {
    if (tags.value.indexOf(tag) < 0) {
      addTags(tag);
    }
  });
}
</script>

<template>
  <v-main>
    <div class="details">
      <div class="map-container">
        <div ref="mapEl" class="map"></div>
      </div>
      <div class="info-panel">
        <h2 class="info-panel-title">{{ selected?.name }}</h2>
        <v-img cover :src="selected.path" @click="photoView = true"></v-img>
        <div class="info-panel-body">
          <v-combobox
            label="Photo Tags"
            :items="tags"
            multiple
            chips
            v-model="selected.tags"
            @update:model-value="updateTags"
          ></v-combobox>
          <v-text-field label="Photo Title" v-model="selected.title"></v-text-field>
          <v-textarea label="Photo Description" v-model="selected.description"></v-textarea>
          <v-checkbox
            label="Location is approximate"
            v-model="selected.locationApprox"
          ></v-checkbox>
          <v-checkbox label="Mark as duplicate" v-model="selected.isDuplicate"></v-checkbox>
        </div>
      </div>
    </div>
    <div class="collection">
      <photo-grid
        :photos="photos"
        :items-per-row="7"
        @select="selectPhoto"
        :size="200"
        :rows="1"
      ></photo-grid>
    </div>
  </v-main>
  <v-dialog v-model="photoView">
    <v-card>
      <v-card-title>{{ selected.name }}</v-card-title>
      <v-card-text>
        <v-img :src="selected.path"></v-img>
      </v-card-text>
    </v-card>
  </v-dialog>
</template>

<style scoped>
.details {
  display: flex;
}

.map-container {
  flex: 2;
}

.map {
  height: 450px;
}

.info-panel {
  height: 450px;
  overflow-y: scroll;
  flex: 1;
}

.info-panel-title {
  margin-left: 8px;
}

.info-panel-body {
  margin: 8px;
}

.photo-grid {
  height: 200px;
}

.collection-control {
  display: inline-block;
}
</style>
