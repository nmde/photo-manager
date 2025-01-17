<script setup lang="ts">
import {
  Chart as ChartJS,
  CategoryScale,
  LineElement,
  LinearScale,
  PointElement,
  Tooltip,
  Title,
  BarElement,
} from 'chart.js';
import { Scatter, Line, Bar } from 'vue-chartjs';
import { fileStore, formatDate } from '../../stores/fileStore';
import SearchInput from '~/components/SearchInput.vue';
import type { Photo } from '~/classes/Photo';

ChartJS.register(CategoryScale, LinearScale, PointElement, LineElement, Title, Tooltip, BarElement);

const {
  people,
  peopleCategories,
  dateMap,
  places,
  layers,
  tags,
  getTagColor,
  journals,
  query,
  search,
} = fileStore;

let targetFiles: Photo[] = [];

const targetFileCount = ref(0);
const targetOptions = ref([
  {
    title: 'People',
    value: 'people',
  },
]);
const typeOptions = ref({
  people: [
    {
      title: 'Count',
      value: 'count',
    },
    {
      title: 'Timeline',
      value: 'timeline',
    },
  ],
});
const precisionOptions = ref([
  {
    title: 'Year',
    value: 0,
  },
  {
    title: 'Month',
    value: 1,
  },
  {
    title: 'Day',
    value: 2,
  },
]);

const graphTarget = ref('');
const graphType = ref('');
const precision = ref(0);
const running = ref(false);
const hasResults = ref(false);

fileStore.on('search', (results) => {
  targetFiles = results;
  targetFileCount.value = results.length;
});

const chartData = ref<{
  labels: string[];
  datasets: any[];
}>({
  labels: [],
  datasets: [],
});

function runCount() {
  const keyMap: Record<string, number> = {};
  // Collect data
  targetFiles.forEach((photo) => {
    if (graphTarget.value === 'people') {
      photo.people.forEach((person) => {
        if (!keyMap[person]) {
          keyMap[person] = 0;
        }
        keyMap[person] += 1;
      });
    }
  });
  const results = Object.entries(keyMap).sort((a, b) => b[1] - a[1]);

  // Generate ChartJS datasets
  const labels: string[] = [];
  const datasets: any[] = [
    {
      label: 'Count',
      backgroundColor: [],
      data: [],
    },
  ];
  results.forEach((entry) => {
    if (graphTarget.value === 'people') {
      const p = people[entry[0]];
      labels.push(p.data.name);
      datasets[0].backgroundColor.push(peopleCategories[p.data.category].data.color);
      datasets[0].data.push(entry[1]);
    }
  });

  chartData.value = {
    labels,
    datasets,
  };
  hasResults.value = true;
  running.value = false;
}

function runTimeline() {
  // Collect data & determine bounds of timeline
  let minDate = new Date();
  let maxDate = new Date(1900, 0, 0);
  const totals: Record<string, number> = {};
  const keyMap: Record<string, Record<string, number>> = {};
  targetFiles
    .filter((photo) => photo.data.date.length > 0)
    .sort((a, b) => {
      if (a.date > b.date) {
        return 1;
      }
      if (a.date < b.date) {
        return -1;
      }
      return 0;
    })
    .forEach((photo) => {
      if (photo.date < minDate) {
        minDate = photo.date;
      }
      if (photo.date > maxDate) {
        maxDate = photo.date;
      }
      const year = photo.date.getFullYear();
      let key = `${year}`;
      if (precision.value >= 1) {
        const month = photo.date.getMonth();
        key = `${year}/${month}`;
        if (precision.value >= 2) {
          key = `${year}/${month}/${photo.date.getDate()}`;
        }
      }
      if (graphTarget.value === 'people') {
        photo.people.forEach((person) => {
          if (!keyMap[key]) {
            keyMap[key] = {};
          }
          if (!totals[person]) {
            totals[person] = 0;
          }
          if (!keyMap[key][person]) {
            keyMap[key][person] = totals[person];
          }
          keyMap[key][person] += 1;
          totals[person] += 1;
        });
      }
    });

  // Fill out timeline
  const labels: string[] = [];
  const timeline: Record<string, number[]> = {};
  let x = 0;
  for (let year = minDate.getFullYear(); year <= maxDate.getFullYear(); year += 1) {
    if (precision.value === 0) {
      const key = `${year}`;
      labels.push(key);
      if (keyMap[key]) {
        Object.entries(keyMap[key]).forEach(([person, value]) => {
          if (!timeline[person]) {
            timeline[person] = [];
          }
          timeline[person][x] = value;
        });
      }
      x += 1;
    } else {
      let month = 0;
      if (year === minDate.getFullYear()) {
        month = minDate.getMonth();
      }
      let stopMonth = 11;
      if (year === maxDate.getFullYear()) {
        stopMonth = maxDate.getMonth();
      }
      for (month; month <= stopMonth; month += 1) {
        if (precision.value === 1) {
          const key = `${year}/${month}`;
          labels.push(key);
          if (keyMap[key]) {
            Object.entries(keyMap[key]).forEach(([person, value]) => {
              if (!timeline[person]) {
                timeline[person] = [];
              }
              timeline[person][x] = value;
            });
          }
          x += 1;
        } else {
          let day = 1;
          if (year === minDate.getFullYear() && month === minDate.getMonth()) {
            day = minDate.getDate();
          }
          let stopDay = new Date(year, month + 1, 0).getDate();
          if (year === maxDate.getFullYear() && month === maxDate.getMonth()) {
            stopDay = maxDate.getDate();
          }
          for (day; day <= stopDay; day += 1) {
            const key = `${year}/${month}/${day}`;
            labels.push(key);
            if (keyMap[key]) {
              Object.entries(keyMap[key]).forEach(([person, value]) => {
                if (!timeline[person]) {
                  timeline[person] = [];
                }
                timeline[person][x] = value;
              });
            }
            x += 1;
          }
        }
      }
    }
  }

  // Generate datasets from timeline
  const datasets: any[] = [];
  console.log(timeline);
  Object.entries(timeline).forEach(([person, data]) => {
    datasets.push({
      label: people[person].data.name,
      backgroundColor: peopleCategories[people[person].data.category].data.color,
      borderColor: peopleCategories[people[person].data.category].data.color,
      data,
      spanGaps: true,
    });
  });

  chartData.value = {
    labels,
    datasets,
  };
  hasResults.value = true;
  running.value = false;
}

onMounted(async () => {
  await search();
});
</script>

<template>
  <v-main>
    <v-container fluid class="fill-height">
      <v-row class="fill-height">
        <v-col cols="8" class="fill-height">
          <template v-if="hasResults">
            <Bar
              v-if="graphType === 'count'"
              :data="chartData"
              :options="{
                maintainAspectRatio: false,
              }"
            ></Bar>
            <Line v-if="graphType === 'timeline'" :data="chartData"></Line>
          </template>
        </v-col>
        <v-col cols="4">
          <SearchInput :value="query"></SearchInput> Computing stats for
          {{ targetFileCount }} photos.
          <v-select
            :disabled="running"
            label="Statistic"
            v-model="graphTarget"
            :items="targetOptions"
          ></v-select>
          <v-select
            v-if="typeOptions[graphTarget]"
            :disabled="running"
            label="Type"
            :items="typeOptions[graphTarget]"
            v-model="graphType"
          ></v-select>
          <v-select
            v-if="graphType === 'timeline'"
            :items="precisionOptions"
            v-model="precision"
            label="Precision"
          ></v-select>
          <v-btn
            :loading="running"
            @click="
              async () => {
                running = true;
                if (graphType === 'count') {
                  runCount();
                } else if (graphType === 'timeline') {
                  runTimeline();
                }
              }
            "
            >Run</v-btn
          >
        </v-col>
      </v-row>
    </v-container>
  </v-main>
</template>

<style scoped>
.fill-height {
  height: 100%;
}
</style>
