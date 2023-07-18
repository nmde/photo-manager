<script setup lang="ts">
import { Loader } from '@googlemaps/js-api-loader';
import { storeToRefs } from 'pinia';
import { computed, ref } from 'vue';
import { Photo } from '../classes/Photo';
import PhotoGrid from '../components/PhotoGrid.vue';
import { useFileStore, stringToLoc, locToString } from '../stores/fileStore';

import { onMounted } from 'vue';

const fileStore = useFileStore();
const { addTags, setLocation } = fileStore;
const { files, tags, locations } = storeToRefs(fileStore);

const selected = ref<Photo[]>([]);
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
    const loc = stringToLoc(pos);
    markers[pos] = new GoogleAdvancedMarkerElement({
      map: map,
      position: loc,
      gmpDraggable: true,
    });
    google.maps.event.addListener(markers[pos], 'click', () => {
      selected.value.forEach((photo) => {
        setLocation(photo.name, loc);
      });
    });
  }
}

/**
 * Select a photo to edit.
 * @param photos - The photo.
 */
function selectPhoto(photos: Photo[]) {
  selected.value = photos;
  if (selected.value.length === 1 && selected.value[0].location !== undefined) {
    placedMarker = true;
    map.setCenter(selected.value[0].location);
  } else {
    // TODO
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
        if (!placedMarker && selected.value.length > 0) {
          createMarker(locToString(e.latLng?.toJSON()));
          placedMarker = true;
          const location = e.latLng?.toJSON();
          selected.value.forEach((photo) => {
            if (location) {
              setLocation(photo.name, location);
            }
          });
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
  selected.value[0].tags.forEach((tag) => {
    if (tags.value.indexOf(tag) < 0) {
      addTags(tag);
    }
  });
}

const photoPath = computed(() => {
  if (selected.value[0].thumbnail) {
    return selected.value[0].thumbnail;
  }
  return selected.value[0].path;
});
</script>

<template>
  <v-main>
    <div class="details">
      <div class="map-container">
        <div ref="mapEl" class="map"></div>
      </div>
      <div class="info-panel" v-if="selected.length > 0">
        <h2 class="info-panel-title">{{ selected[0].name }}</h2>
        <v-img cover :src="photoPath" @click="photoView = true"></v-img>
        <div class="info-panel-body">
          <v-combobox
            label="Photo Tags"
            :items="tags"
            multiple
            chips
            v-model="selected[0].tags"
            @update:model-value="updateTags"
          ></v-combobox>
          <v-text-field label="Photo Title" v-model="selected[0].title"></v-text-field>
          <v-textarea label="Photo Description" v-model="selected[0].description"></v-textarea>
          <v-checkbox
            label="Location is approximate"
            v-model="selected[0].locationApprox"
          ></v-checkbox>
          <v-checkbox label="Mark as duplicate" v-model="selected[0].isDuplicate"></v-checkbox>
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
      <v-card-title>{{ selected[0].name }}</v-card-title>
      <v-card-text>
        <v-img :src="photoPath"></v-img>
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
