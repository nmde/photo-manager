<script setup lang="ts">
import { Loader } from '@googlemaps/js-api-loader';
import { storeToRefs } from 'pinia';
import { onMounted, ref, computed } from 'vue';
import { Photo, createPhoto } from '../classes/Photo';
import PhotoIcon from '../components/PhotoIcon.vue';
import { useFileStore, stringToLoc } from '../stores/fileStore';

const { locations, tags, files } = storeToRefs(useFileStore());

const tagMap = ref<Record<string, boolean>>({});
const mapEl = ref(null);
const filterBy = ref(0); // 0 - tags, 1 - location
const filterPos = ref<{ lat: number; lng: number }>({ lat: 0, lng: 0 });

/**
 * Toggles a tag.
 * @param tag - The tag to toggle.
 */
function toggle(tag: string) {
  filterBy.value = 0;
  tagMap.value[tag] = !tagMap.value[tag];
}

/**
 * Sets the value of all tags.
 * @param to - The value to set to.
 */
function toggleAll(to: boolean) {
  filterBy.value = 0;
  Object.keys(tagMap.value).forEach((key) => {
    tagMap.value[key] = to;
  });
}

const filteredPhotos = computed(() => {
  const filtered: Record<string, Photo> = {};
  if (filterBy.value === 0) {
    const enabledTags: string[] = [];
    Object.entries(tagMap.value).forEach(([tag, enabled]) => {
      if (enabled) {
        enabledTags.push(tag);
      }
    });
    Object.values(files.value).forEach((file) => {
      file.tags.forEach((tag) => {
        if (enabledTags.indexOf(tag) >= 0) {
          filtered[file.name] = file;
        }
      });
    });
  } else if (filterBy.value === 1) {
    Object.values(files.value).forEach((file) => {
      if (file.location) {
        if (
          file.location.lat === filterPos.value.lat &&
          file.location.lng === filterPos.value.lng
        ) {
          filtered[file.name] = file;
        }
      }
    });
  }
  return filtered;
});

const photoView = ref(false);
const selected = ref<Photo>(createPhoto('', ''));

/**
 * Opens the single photo view.
 * @param photo - The photo to open for.
 */
function view(photo: string) {
  selected.value = files.value[photo];
  photoView.value = true;
}

const markers: Record<string, google.maps.marker.AdvancedMarkerElement> = {};

onMounted(() => {
  tags.value.forEach((tag) => {
    tagMap.value[tag] = true;
  });
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

      const map = new Map(mapEl.value as unknown as HTMLElement, {
        zoom: 6,
        mapId: 'DEMO_MAP_ID',
      });

      navigator.geolocation.getCurrentPosition((position: GeolocationPosition) => {
        map.setCenter({
          lat: position.coords.latitude,
          lng: position.coords.longitude,
        });
      });

      const createMarker = (pos: string) => {
        if (!markers[pos]) {
          const position = stringToLoc(pos);
          markers[pos] = new AdvancedMarkerElement({
            map: map,
            position,
          });
          google.maps.event.addListener(markers[pos], 'click', () => {
            filterBy.value = 1;
            filterPos.value = position;
          });
        }
      };

      Object.entries(locations.value).forEach(([loc]) => {
        createMarker(loc);
      });
    });
});
</script>

<template>
  <v-main>
    <v-container fluid>
      <v-row>
        <v-col cols="6">
          <v-btn @click="toggleAll(true)">Select All</v-btn>
          <v-btn @click="toggleAll(false)">Deselect All</v-btn>
          <br />
          <v-btn
            v-for="(enabled, tag) in tagMap"
            :key="tag"
            :color="enabled ? 'primary' : ''"
            @click="toggle(tag)"
            >{{ tag }}</v-btn
          >
          <div class="photo-grid">
            <photo-icon
              v-for="(photo, i) in filteredPhotos"
              :key="i"
              :photo="photo"
              :size="240"
              @select="view(i)"
            ></photo-icon>
          </div>
        </v-col>
        <v-col cols="6">
          <div class="map" ref="mapEl"></div>
        </v-col>
      </v-row>
    </v-container>
    <v-dialog v-model="photoView">
      <v-card>
        <v-card-title>{{ selected.name }}</v-card-title>
        <v-card-text>
          <v-img max-height="600" :src="selected.path"></v-img>
          Title: {{ selected.title }} <br />
          Description: {{ selected.description }} <br />
          Tags: {{ selected.tags.join(',') }}
        </v-card-text>
      </v-card>
    </v-dialog>
  </v-main>
</template>

<style scoped>
.map {
  height: 450px;
}
</style>
