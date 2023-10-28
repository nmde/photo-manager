<script setup lang="ts">
import { Chart as ChartJS, BarElement, CategoryScale, LinearScale, Tooltip } from 'chart.js';
import { storeToRefs } from 'pinia';
import { computed, ref } from 'vue';
import { Bar } from 'vue-chartjs';
import { Photo } from '../../classes/Photo';
import { useFileStore } from '../../stores/fileStore';

/**
 * TODO:
 * - Delete a tag
 * - View redundant tags
 * - Tag network
 * - Tag influence on rating
 */

ChartJS.register(CategoryScale, LinearScale, BarElement, Tooltip);

const fileStore = useFileStore();
const { files, tags, tagCounts } = storeToRefs(fileStore);

const selected = ref<Photo[]>([]);
const enabledTags = ref<string[]>([]);
const disabledTags = ref<string[]>([]);
const searchDialog = ref(false);
const includeMode = ref('AND');

const photos = computed(() => {
  const filtered: Photo[] = [];
  console.log(disabledTags.value);
  Object.values(files.value).forEach((file) => {
    let satisfiesTags = includeMode.value === 'AND' || enabledTags.value.length === 0;
    enabledTags.value.forEach((tag) => {
      if (includeMode.value === 'OR' && file.tags.indexOf(tag) >= 0) {
        satisfiesTags = true;
      } else if (includeMode.value === 'AND' && file.tags.indexOf(tag) < 0) {
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
  return filtered;
});

const tagChartData = computed(() => {
  let sorted: string[] = [];
  const cutoff = 2;
  Object.entries(tagCounts.value)
    .filter((count) => count[1] >= cutoff)
    .forEach(([tag, value]) => {
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
        label: 'Count',
        data: sorted.map((tag) => tagCounts.value[tag]),
      },
    ],
  };
});
</script>

<template>
  <v-main class="main">
    <v-container fluid>
      <v-row>
        <v-col cols="6">
          <div class="flex">
            <v-combobox
              label="Tags to include"
              :items="tags"
              multiple
              chips
              clearable
              v-model="enabledTags"
            >
            </v-combobox>
            <v-btn @click="searchDialog = true">Advanced</v-btn>
          </div>
          <photo-grid
            :photos="photos"
            :items-per-row="4"
            @select="(s) => (selected = s)"
            :size="175"
            :rows="4"
          ></photo-grid>
        </v-col>
        <v-col cols="6">
          <photo-group v-if="selected.length > 0" :photos="selected"></photo-group>
          <Bar
            :options="{
              indexAxis: 'y',
            }"
            :data="tagChartData"
          ></Bar>
        </v-col>
      </v-row>
    </v-container>
    <v-dialog v-model="searchDialog">
      <v-card>
        <v-card-text>
          <v-combobox
            label="Tags to include"
            :items="tags"
            multiple
            chips
            clearable
            v-model="enabledTags"
          >
          </v-combobox>
          <v-select :items="['AND', 'OR']" label="Mode" v-model="includeMode"></v-select>
          <v-combobox
            label="Tags to exclude"
            :items="tags"
            multiple
            chips
            clearable
            v-model="disabledTags"
          >
          </v-combobox>
        </v-card-text>
      </v-card>
    </v-dialog>
  </v-main>
</template>

<style scoped></style>
