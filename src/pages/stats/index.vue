<script setup lang="ts">
import {
  Chart as ChartJS,
  CategoryScale,
  LineElement,
  LinearScale,
  PointElement,
  Tooltip,
  Title,
} from 'chart.js';
import { Line } from 'vue-chartjs';
import { fileStore, formatDate } from '../../stores/fileStore';

ChartJS.register(CategoryScale, LinearScale, PointElement, LineElement, Title, Tooltip);

const { people, peopleCategories, firstDate, lastDate, dateMap } = fileStore;

const enabledPeople = ref<Record<string, boolean>>({});

const peopleTimeline = computed(() => {
  const labels: string[] = [];
  const datasets: any[] = [];
  const totals: Record<string, number> = {};
  const timeline: Record<string, Record<string, Record<string, Record<string, number>>>> = {};
  Object.entries(people).forEach(([id, person]) => {
    if (enabledPeople.value[id]) {
      totals[id] = 0;
      datasets.push({
        label: person.data.name,
        data: [],
        backgroundColor: peopleCategories[person.data.category].data.color,
        borderColor: peopleCategories[person.data.category].data.color,
        spanGaps: true,
      });
    }
  });
  let x = 0;
  for (let year = firstDate.getFullYear(); year <= lastDate.getFullYear(); year += 1) {
    timeline[year] = {};
    let month = 0;
    if (year === firstDate.getFullYear()) {
      month = firstDate.getMonth();
    }
    let stopMonth = 11;
    if (year === lastDate.getFullYear()) {
      stopMonth = lastDate.getMonth();
    }
    for (month; month <= stopMonth; month += 1) {
      timeline[year][month] = {};
      let day = 1;
      if (year === firstDate.getFullYear() && month === firstDate.getMonth()) {
        day = firstDate.getDate();
      }
      let stopDay = new Date(year, month + 1, 0).getDate();
      if (year === lastDate.getFullYear() && month === lastDate.getMonth()) {
        stopDay = lastDate.getDate();
      }
      for (day; day <= stopDay; day += 1) {
        const d = formatDate(new Date(year, month, day));
        labels.push(d);
        timeline[year][month][day] = {};
        if (dateMap[d]) {
          const photos = dateMap[d];
          let i = 0;
          Object.keys(people).forEach((id) => {
            if (enabledPeople.value[id]) {
              const count = photos.filter((f) => f.people.indexOf(id) >= 0).length;
              if (count > 0) {
                totals[id] += count;
                datasets[i].data[x] = totals[id];
              }
              i += 1;
            }
          });
        }
        x += 1;
      }
    }
  }
  return {
    labels,
    datasets,
  };
});

onMounted(() => {
  Object.keys(people).forEach((id) => {
    enabledPeople.value[id] = true;
  });
});
</script>

<template>
  <v-main>
    <Line :data="peopleTimeline"></Line>
    <div>
      <v-checkbox
        v-for="person in people"
        :key="person.Id"
        :label="person.data.name"
        v-model="enabledPeople[person.Id]"
      ></v-checkbox>
    </div>
  </v-main>
</template>
