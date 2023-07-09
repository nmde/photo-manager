<script setup lang="ts">
import { Loader } from '@googlemaps/js-api-loader';
import { writeTextFile } from '@tauri-apps/api/fs';
import { join } from '@tauri-apps/api/path';
import { storeToRefs } from 'pinia';
import { ref } from 'vue';
import { Photo, createPhoto } from '../classes/Photo';
import PhotoIcon from '../components/PhotoIcon.vue';
import { useFileStore, stringToLoc, locToString } from '../stores/fileStore';
import { PhotoDataFile } from '../types/photo-data';
import { onMounted } from 'vue';

const { files, workingDir, tags, locations } = storeToRefs(useFileStore());

const showOnlyUntagged = ref(true);
const selected = ref<Photo>(createPhoto('', ''));
const hasSelected = ref(false);
const mapEl = ref(null);
const saved = ref(false);

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

const saving = ref(false);

/**
 * Saves the photo data.
 */
async function save() {
  saving.value = true;
  const photoManagerFile = await join(workingDir.value, 'photo-data.json');
  const photoData: PhotoDataFile = {
    files: files.value,
    tags: tags.value,
    locations: {},
  };
  await writeTextFile(photoManagerFile, JSON.stringify(photoData));
  saving.value = false;
  saved.value = true;
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
</script>

<template>
  <v-main>
    <v-toolbar>
      <v-toolbar-title>{{ workingDir }}</v-toolbar-title>
      <v-spacer></v-spacer>
      <v-btn color="primary" @click="save" :loading="saving">Save</v-btn>
    </v-toolbar>
    <div class="details">
      <div class="map-container">
        <div ref="mapEl" class="map"></div>
      </div>
      <div class="info-panel">
        <h2 class="info-panel-title">{{ selected?.name }}</h2>
        <v-img :src="selected.path"></v-img>
        <div class="info-panel-body">
          <v-combobox
            label="Photo Tags"
            :items="tags"
            multiple
            chips
            v-model="selected.tags"
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
      <v-checkbox v-model="showOnlyUntagged" label="Show only untagged"></v-checkbox>
      <div class="photo-grid">
        <photo-icon
          v-for="(photo, i) in files"
          :key="i"
          :photo="photo"
          :size="200"
          @select="selectPhoto(photo)"
        ></photo-icon>
      </div>
    </div>
  </v-main>
  <v-snackbar v-model="saved">Collection saved</v-snackbar>
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
  overflow-y: scroll;
}
</style>
