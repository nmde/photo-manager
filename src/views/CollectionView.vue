<script setup lang="ts">
import { Chart as ChartJS, BarElement, CategoryScale, LinearScale } from 'chart.js';
import { storeToRefs } from 'pinia';
import { onMounted, ref, computed } from 'vue';
import { Bar } from 'vue-chartjs';
import { Map, Position } from '../classes/Map';
import { Photo, createPhoto } from '../classes/Photo';
import PhotoDetail from '../components/PhotoDetail.vue';
import PhotoGrid from '../components/PhotoGrid.vue';
import PhotoGroup from '../components/PhotoGroup.vue';
import { useFileStore } from '../stores/fileStore';

ChartJS.register(CategoryScale, LinearScale, BarElement);

const { locations, tags, files, tagCounts } = storeToRefs(useFileStore());

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
  let sorted: string[] = [];
  console.log(tagCounts.value);
  Object.entries(tagCounts.value).forEach(([tag, value]) => {
    if (sorted.length === 0) {
      sorted.push(tag);
    } else {
      let i = 0;
      while (i < sorted.length && value < tagCounts.value[sorted[i]]) {
        i += 1;
      }
      sorted.splice(i, 0, tag);
    }
  });
  return {
    labels: sorted,
    datasets: [
      {
        axis: 'y',
        labebl: 'Tag Counts',
        data: sorted.map((tag) => tagCounts.value[tag]),
      },
    ],
  };
});

const displayName = computed(() => {
  if (selected.value.group !== undefined) {
    return selected.value.group;
  }
  return selected.value.name;
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
        <v-card-title>{{ displayName }}</v-card-title>
        <v-card-text>
          <photo-detail :photo="selected" v-if="selected.group === undefined"></photo-detail>
          <photo-group :group="selected.group" v-if="selected.group !== undefined"></photo-group>
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
