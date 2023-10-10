<script setup lang="ts">
import { Chart as ChartJS, BarElement, CategoryScale, LinearScale, Tooltip } from 'chart.js';
import { storeToRefs } from 'pinia';
import { computed, ref } from 'vue';
import { Bar } from 'vue-chartjs';
import { Photo } from '../../classes/Photo';
import { useFileStore } from '../../stores/fileStore';

ChartJS.register(CategoryScale, LinearScale, BarElement, Tooltip);

const fileStore = useFileStore();
const { files, tags, tagCounts } = storeToRefs(fileStore);

const selected = ref<Photo[]>([]);
const enabledTags = ref<string[]>([]);

const photos = computed(() => {
  const filtered: Photo[] = [];
  Object.values(files.value).forEach((file) => {
    let satisfiesTags = enabledTags.value.length === 0;
    enabledTags.value.forEach((tag) => {
      if (file.tags.indexOf(tag) >= 0) {
        satisfiesTags = true;
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
          <v-combobox
            label="Tags to include"
            :items="tags"
            multiple
            chips
            clearable
            v-model="enabledTags"
          >
          </v-combobox>
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
  </v-main>
</template>

<style scoped></style>
