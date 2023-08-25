<script setup lang="ts">
import { Chart as ChartJS, BarElement, CategoryScale, LinearScale } from 'chart.js'
import { storeToRefs } from 'pinia';
import { onMounted, ref, computed } from 'vue';
import { Bar } from 'vue-chartjs';
import { Map, Position } from '../classes/Map';
import { Photo, createPhoto } from '../classes/Photo';
import PhotoGrid from '../components/PhotoGrid.vue';
import { useFileStore } from '../stores/fileStore';

ChartJS.register(CategoryScale, LinearScale, BarElement);

const fileStore = useFileStore();
const { addTags } = fileStore;
const { locations, tags, files } = storeToRefs(fileStore);

const mapEl = ref(null);
const filterBy = ref(0); // 0 - tags, 1 - location
const filterPos = ref<Position>({ lat: 0, lng: 0 });
const enabledTags = ref<string[]>([]);
const disabledTags = ref<string[]>([]);
const showHeatmap = ref(false);

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
 * @param photos - The photo to open for.
 */
function view(photos: Photo[]) {
  // TODO: open a gallery for multiple photos
  selected.value = photos[0];
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

const map = new Map();
onMounted(async () => {
  await map.initialize(mapEl.value as unknown as HTMLElement);
  Object.entries(locations.value).forEach(([loc, count]) => {
    map.createMarker(loc, count);
  });
  map.createHeatmap();
  map.on('markerClicked', (pos) => {
    filterBy.value = 1;
    filterPos.value = pos;
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

const tagChartData = computed(() => {
  const map: Record<string, number> = {};
  Object.values(files.value).forEach((file) => {
    file.tags.forEach((tag) => {
      if (!map[tag]) {
        map[tag] = 0;
      }
      map[tag] += 1;
    });
  });
  return {
    labels: Object.keys(map),
    datasets: [
      {
        axis: 'y',
        labebl: 'Tag Counts',
        data: Object.values(map),
      },
    ],
  };
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
          <v-checkbox
            label="Show heatmap"
            v-model="showHeatmap"
            @update:model-value="toggleHeatmap()"
          ></v-checkbox>
          <Bar :options="{ indexAxis: 'y' }" :data="tagChartData"></Bar>
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
          <v-rating v-model="selected.rating"></v-rating>
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
