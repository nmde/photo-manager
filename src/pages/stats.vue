<script setup lang="ts">
  import type { Photo } from '../classes/Photo';
  import {
    BarElement,
    CategoryScale,
    Chart as ChartJS,
    LinearScale,
    LineElement,
    PointElement,
    Title,
    Tooltip,
  } from 'chart.js';
  import { onMounted, ref } from 'vue';
  import { Bar, Line } from 'vue-chartjs';
  import { fileStore } from '../stores/fileStore';

  ChartJS.register(
    CategoryScale,
    LinearScale,
    PointElement,
    LineElement,
    Title,
    Tooltip,
    BarElement,
  );

  const { people, peopleCategories, query, search } = fileStore;

  let targetFiles: Photo[] = [];

  type Target = {
    title: string;
    value: string | number;
  };

  const targetOptions = ref<Target[]>([
    {
      title: 'People',
      value: 'people',
    },
  ]);
  const typeOptions = ref<Record<string, Target[]>>({
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
  const precisionOptions = ref<Target[]>([
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

  fileStore.on('search', results => {
    targetFiles = results;
  });

  const chartData = ref<{
    labels: string[];
    datasets: any[];
  }>({
    labels: [],
    datasets: [],
  });

  type Dataset = {
    label: string;
    backgroundColor: string[];
    data: any[];
  };

  function runCount() {
    const keyMap: Record<string, number> = {};
    // Collect data
    for (const photo of targetFiles) {
      if (graphTarget.value === 'people') {
        for (const person of photo.people) {
          if (!keyMap[person]) {
            keyMap[person] = 0;
          }
          keyMap[person] += 1;
        }
      }
    }
    const results = Object.entries(keyMap).toSorted((a, b) => b[1] - a[1]);

    // Generate ChartJS datasets
    const labels: string[] = [];
    const datasets: Dataset[] = [
      {
        label: 'Count',
        backgroundColor: [],
        data: [],
      },
    ];
    for (const entry of results) {
      if (graphTarget.value === 'people') {
        const p = people[entry[0]];
        if (p) {
          labels.push(p.name);
          datasets[0]?.backgroundColor.push(peopleCategories[p.category]?.color ?? '');
          datasets[0]?.data.push(entry[1]);
        }
      }
    }

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
    for (const photo of targetFiles
      .filter(photo => photo.hasDate)
      .toSorted((a, b) => {
        if (a.date > b.date) {
          return 1;
        }
        if (a.date < b.date) {
          return -1;
        }
        return 0;
    })) {
      if (photo.date < minDate) {
        minDate = photo.date;
      }
      if (photo.date > maxDate) {
        maxDate = photo.date;
      }
      const year = photo.date.getFullYear();
      let key = year.toString();
      if (precision.value >= 1) {
        const month = photo.date.getMonth();
        key = `${year.toString()}/${month.toString()}`;
        if (precision.value >= 2) {
          key = `${year.toString()}/${month.toString()}/${photo.date.getDate().toString()}`;
        }
      }
      if (graphTarget.value === 'people') {
        for (const person of photo.people) {
          if (!keyMap[key]) {
            keyMap[key] = {};
          }
          if (!totals[person]) {
            totals[person] = 0;
          }
          if (!keyMap[key]?.[person]) {
            keyMap[key][person] = totals[person];
          }
          keyMap[key][person] += 1;
          totals[person] += 1;
        }
      }
    }

    // Fill out timeline
    const labels: string[] = [];
    const timeline: Record<string, number[]> = {};
    let x = 0;
    for (let year = minDate.getFullYear(); year <= maxDate.getFullYear(); year += 1) {
      if (precision.value === 0) {
        const key = year.toString();
        labels.push(key);
        if (keyMap[key]) {
          for (const [person, value] of Object.entries(keyMap[key])) {
            if (!timeline[person]) {
              timeline[person] = [];
            }
            timeline[person][x] = value;
          }
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
            const key = `${year.toString()}/${month.toString()}`;
            labels.push(key);
            if (keyMap[key]) {
              for (const [person, value] of Object.entries(keyMap[key])) {
                if (!timeline[person]) {
                  timeline[person] = [];
                }
                timeline[person][x] = value;
              }
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
              const key = `${year.toString()}/${month.toString()}/${day.toString()}`;
              labels.push(key);
              if (keyMap[key]) {
                for (const [person, value] of Object.entries(keyMap[key])) {
                  if (!timeline[person]) {
                    timeline[person] = [];
                  }
                  timeline[person][x] = value;
                }
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
    for (const [person, data] of Object.entries(timeline)) {
      const c = people[person]?.category;
      if (c) {
        datasets.push({
          label: people[person]?.name,
          backgroundColor: peopleCategories[c]?.color,
          borderColor: peopleCategories[c]?.color,
          data,
          spanGaps: true,
        });
      }
    }

    chartData.value = {
      labels,
      datasets,
    };
    hasResults.value = true;
    running.value = false;
  }

  onMounted(() => {
    search([]);
  });
</script>

<template>
  <v-main>
    <v-container class="fill-height" fluid>
      <v-row class="fill-height">
        <v-col class="fill-height" cols="8">
          <template v-if="hasResults">
            <Bar
              v-if="graphType === 'count'"
              :data="chartData"
              :options="{
                maintainAspectRatio: false,
              }"
            />
            <Line v-if="graphType === 'timeline'" :data="chartData" />
          </template>
        </v-col>
        <v-col cols="4">
          <SearchInput :value="query" /> Computing stats for {{ targetFiles.length }} photos.
          <v-select
            v-model="graphTarget"
            :disabled="running"
            :items="targetOptions"
            label="Statistic"
          />
          <v-select
            v-if="typeOptions[graphTarget]"
            v-model="graphType"
            :disabled="running"
            :items="typeOptions[graphTarget]"
            label="Type"
          />
          <v-select
            v-if="graphType === 'timeline'"
            v-model="precision"
            :items="precisionOptions"
            label="Precision"
          />
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
          >
            Run
          </v-btn>
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
