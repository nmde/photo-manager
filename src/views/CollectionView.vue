<script setup lang="ts">
import { Loader } from '@googlemaps/js-api-loader';
import { storeToRefs } from 'pinia';
import { ref } from 'vue';
import { Photo } from '../classes/Photo';
import PhotoIcon from '../components/PhotoIcon.vue';
import { useFileStore } from '../stores/fileStore';
import { onMounted } from 'vue';

const { files, workingDir } = storeToRefs(useFileStore());

const showOnlyUntagged = ref(true);
const selected = ref<Photo | null>(null);
const hasSelected = ref(false);
const mapEl = ref(null);

let map: google.maps.Map;
let placedMarker = false;

/**
 * Select a photo to edit.
 * @param photo - The photo.
 */
function selectPhoto(photo: Photo) {
  selected.value = photo;
  hasSelected.value = true;
  placedMarker = false; // TODO - check if the photo has a location already
}

onMounted(() => {
  new Loader({
    apiKey: import.meta.env.VITE_GOOGLE_MAPS_KEY,
    version: 'weekly',
  })
    .load()
    .then(async () => {
      const { Map, InfoWindow } = (await google.maps.importLibrary(
        'maps',
      )) as google.maps.MapsLibrary;
      const { AdvancedMarkerElement } = (await google.maps.importLibrary(
        'marker',
      )) as google.maps.MarkerLibrary;

      map = new Map(mapEl.value as unknown as HTMLElement, {
        zoom: 6,
        mapId: 'DEMO_MAP_ID',
      });

      navigator.geolocation.getCurrentPosition((position: GeolocationPosition) => {
        const pos = {
          lat: position.coords.latitude,
          lng: position.coords.longitude,
        };
        map.setCenter(pos);
      });

      const infoWindow = new InfoWindow();

      map.addListener('dblclick', (e: google.maps.MapMouseEvent) => {
        if (!placedMarker && hasSelected) {
          const marker = new AdvancedMarkerElement({
            map: map,
            position: e.latLng?.toJSON(),
            title: selected.value?.name,
            gmpDraggable: true,
          });
          google.maps.event.addListener(marker, 'click', () => {
            infoWindow.close();
            infoWindow.setContent(selected.value?.name);
            infoWindow.open(map, marker);
          });
          placedMarker = true;
        }
      });
    });
});
</script>

<template>
  <v-main>
    <v-container fluid>
      <v-row>
        <v-col cols="6">
          <h1>{{ workingDir }}</h1>
          <v-checkbox v-model="showOnlyUntagged" label="Show only untagged"></v-checkbox>
        </v-col>
        <v-col cols="6">
          <h1>{{ selected?.name }}</h1>
        </v-col>
      </v-row>
      <v-row>
        <v-col cols="6">
          <photo-icon
            v-for="(photo, i) in files"
            :key="i"
            :photo="photo"
            :size="150"
            @select="selectPhoto(photo)"
          ></photo-icon>
        </v-col>
        <v-col cols="6" class="map-container">
          <v-img v-if="hasSelected" :src="selected?.path"></v-img>
          <div ref="mapEl" class="map"></div>
        </v-col>
      </v-row>
    </v-container>
  </v-main>
</template>

<style scoped>
.map-container {
  height: 500px;
}

.map {
  height: 100%;
}
</style>
