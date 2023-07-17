<script setup lang="ts">
import { Loader } from '@googlemaps/js-api-loader';
import { storeToRefs } from 'pinia';
import { onMounted, ref, computed } from 'vue';
import { Photo, createPhoto } from '../classes/Photo';
import PhotoGrid from '../components/PhotoGrid.vue';
import { useFileStore, stringToLoc } from '../stores/fileStore';

const fileStore = useFileStore();
const { addTags } = fileStore;
const { locations, tags, files } = storeToRefs(fileStore);

const mapEl = ref(null);
const filterBy = ref(0); // 0 - tags, 1 - location
const filterPos = ref<{ lat: number; lng: number }>({ lat: 0, lng: 0 });
const enabledTags = ref<string[]>([]);
const disabledTags = ref<string[]>([]);

const filteredPhotos = computed(() => {
  const filtered: Photo[] = [];
  if (filterBy.value === 0) {
    Object.values(files.value).forEach((file) => {
      let satisfiesTags = true;
      enabledTags.value.forEach((tag) => {
        if (file.tags.indexOf(tag) < 0) {
          satisfiesTags = false;
        }
      });
      disabledTags.value.forEach((tag) => {
        if (file.tags.indexOf(tag) >= 0) {
          satisfiesTags = false;
        }
      });
      if (satisfiesTags) {
        filtered.push(file);
      }
    });
  } else if (filterBy.value === 1) {
    Object.values(files.value).forEach((file) => {
      if (file.location) {
        if (
          file.location.lat === filterPos.value.lat &&
          file.location.lng === filterPos.value.lng
        ) {
          filtered.push(file);
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
function view(photo: Photo) {
  selected.value = photo;
  photoView.value = true;
}

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

const photoPath = computed(() => {
  if (selected.value.thumbnail) {
    return selected.value.thumbnail;
  }
  return selected.value.path;
});

const markers: Record<string, google.maps.marker.AdvancedMarkerElement> = {};

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
          <v-combobox
            label="Tags to include"
            :items="tags"
            multiple
            chips
            clearable
            v-model="enabledTags"
            @update:model-value="filterBy = 0"
          >
          </v-combobox>
          <v-combobox
            label="Tags to exclude"
            :items="tags"
            multiple
            chips
            v-model="disabledTags"
            clearable
            @update:model-value="filterBy = 0"
          ></v-combobox>
          <photo-grid
            :photos="filteredPhotos"
            :items-per-row="3"
            :rows="3"
            :size="230"
            @select="view"
          ></photo-grid>
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
          <v-img max-height="600" :src="photoPath"></v-img>
          Title: {{ selected.title }} <br />
          Description: {{ selected.description }} <br />
          <v-combobox
            label="Photo Tags"
            :items="tags"
            multiple
            chips
            v-model="selected.tags"
            @update:model-value="updateTags"
          ></v-combobox>
          <v-checkbox label="Mark as duplicate" v-model="selected.isDuplicate"></v-checkbox>
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
