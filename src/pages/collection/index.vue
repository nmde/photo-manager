<script setup lang="ts">
import { Loader } from '@googlemaps/js-api-loader';
import { storeToRefs } from 'pinia';
import { computed, ref, onMounted } from 'vue';
import { stringToLoc, locToString } from '../../classes/Map';
import { Photo } from '../../classes/Photo';
import { useFileStore } from '../../stores/fileStore';

const config = useRuntimeConfig();

const fileStore = useFileStore();
const { setLocation } = fileStore;
const { files, locations } = storeToRefs(fileStore);

const selected = ref<Photo[]>([]);
const mapEl = ref(null);

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
        setLocation(photo.data.name, loc);
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
    apiKey: config.public.GOOGLE_MAPS_KEY as string,
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
              setLocation(photo.data.name, location);
            }
          });
        }
      });

      Object.entries(locations.value)
        .filter((loc) => loc[0].length > 1)
        .forEach(([loc]) => {
          createMarker(loc);
        });
    });
});
</script>

<template>
  <v-main>
    <div class="map-container">
      <div ref="mapEl" class="map"></div>
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
</template>

<style scoped>
.map-container {
  flex: 2;
}

.map {
  height: 450px;
}
</style>
