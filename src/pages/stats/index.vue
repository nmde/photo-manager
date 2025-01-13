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
import { Scatter, Line } from 'vue-chartjs';
import { fileStore, formatDate } from '../../stores/fileStore';

ChartJS.register(CategoryScale, LinearScale, PointElement, LineElement, Title, Tooltip);

const {
  people,
  peopleCategories,
  firstDate,
  lastDate,
  dateMap,
  places,
  layers,
  tags,
  getTagColor,
  journals,
} = fileStore;

const graphType = ref('');
const graphOptions = ref([
  {
    title: 'People',
    value: 'people',
  },
  {
    title: 'Photographers',
    value: 'photographers',
  },
  {
    title: 'Places',
    value: 'places',
  },
  {
    title: 'Tags',
    value: 'tags',
  },
  {
    title: 'Moods',
    value: 'moods',
  },
]);

const enabledPeople = ref<Record<string, boolean>>({});
const enabledPlaces = ref<Record<string, boolean>>({});

const peopleTimeline = computed(() => {
  const labels: string[] = [];
  const datasets: any[] = [];
  const totals: Record<string, number> = {};
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
    let month = 0;
    if (year === firstDate.getFullYear()) {
      month = firstDate.getMonth();
    }
    let stopMonth = 11;
    if (year === lastDate.getFullYear()) {
      stopMonth = lastDate.getMonth();
    }
    for (month; month <= stopMonth; month += 1) {
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

const photographerTimeline = computed(() => {
  const labels: string[] = [];
  const datasets: any[] = [];
  const totals: Record<string, number> = {};
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
    let month = 0;
    if (year === firstDate.getFullYear()) {
      month = firstDate.getMonth();
    }
    let stopMonth = 11;
    if (year === lastDate.getFullYear()) {
      stopMonth = lastDate.getMonth();
    }
    for (month; month <= stopMonth; month += 1) {
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
        if (dateMap[d]) {
          const photos = dateMap[d];
          let i = 0;
          Object.keys(people).forEach((id) => {
            if (enabledPeople.value[id]) {
              const count = photos.filter((f) => f.data.photographer === id).length;
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

const placesTimeline = computed(() => {
  const labels: string[] = [];
  const datasets: any[] = [];
  const totals: Record<string, number> = {};
  Object.entries(places).forEach(([id, place]) => {
    if (enabledPlaces.value[id]) {
      totals[id] = 0;
      datasets.push({
        label: place.data.name,
        data: [],
        backgroundColor: layers[place.data.layer].data.color,
        borderColor: layers[place.data.layer].data.color,
        spanGaps: true,
      });
    }
  });
  let x = 0;
  for (let year = firstDate.getFullYear(); year <= lastDate.getFullYear(); year += 1) {
    let month = 0;
    if (year === firstDate.getFullYear()) {
      month = firstDate.getMonth();
    }
    let stopMonth = 11;
    if (year === lastDate.getFullYear()) {
      stopMonth = lastDate.getMonth();
    }
    for (month; month <= stopMonth; month += 1) {
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
        if (dateMap[d]) {
          const photos = dateMap[d];
          let i = 0;
          Object.keys(places).forEach((id) => {
            if (enabledPlaces.value[id]) {
              const count = photos.filter((f) => f.data.location === id).length;
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

const tagsTimeline = computed(() => {
  const labels: string[] = [];
  const datasets: any[] = [];
  const totals: Record<string, number> = {};
  tags.forEach((tag) => {
    totals[tag] = 0;
    const color = getTagColor(tag);
    datasets.push({
      label: tag,
      data: [],
      backgroundColor: color,
      borderColor: color,
      spanGaps: true,
    });
  });
  let x = 0;
  for (let year = firstDate.getFullYear(); year <= lastDate.getFullYear(); year += 1) {
    let month = 0;
    if (year === firstDate.getFullYear()) {
      month = firstDate.getMonth();
    }
    let stopMonth = 11;
    if (year === lastDate.getFullYear()) {
      stopMonth = lastDate.getMonth();
    }
    for (month; month <= stopMonth; month += 1) {
      labels.push(`${year}/${month}`);
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
        if (dateMap[d]) {
          const photos = dateMap[d];
          let i = 0;
          tags.forEach((tag) => {
            const count = photos.filter((f) => f.hasTag(tag)).length;
            if (count > 0) {
              totals[tag] += count;
              if (datasets[i].data[x]) {
                datasets[i].data[x] += count;
              } else {
                datasets[i].data[x] = totals[tag];
              }
            }
            i += 1;
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

const moodTimeline = computed(() => {
  const labels: string[] = [];
  const datasets: any[] = [
    {
      label: 'Awful',
      data: [],
      backgroundColor: '#F44336',
      borderColor: '#F44336',
    },
    {
      label: 'Bad',
      data: [],
      backgroundColor: '#FF9800',
      borderColor: '#FF9800',
    },
    {
      label: 'Meh',
      data: [],
      backgroundColor: '#2196F3',
      borderColor: '#2196F3',
    },
    {
      label: 'Good',
      data: [],
      backgroundColor: '#4CAF50',
      borderColor: '#4CAF50',
    },
    {
      label: 'Awesome',
      data: [],
      backgroundColor: '#009688',
      borderColor: '#009688',
    },
  ];
  let x = 0;
  for (let year = firstDate.getFullYear(); year <= lastDate.getFullYear(); year += 1) {
    let month = 0;
    if (year === firstDate.getFullYear()) {
      month = firstDate.getMonth();
    }
    let stopMonth = 11;
    if (year === lastDate.getFullYear()) {
      stopMonth = lastDate.getMonth();
    }
    for (month; month <= stopMonth; month += 1) {
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
        if (journals[d]) {
          labels.push(`${year}/${month}/${day}`);
          datasets[journals[d].data.mood].data.push({
            x,
            y: journals[d].data.mood,
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
  Object.keys(places).forEach((id) => {
    enabledPlaces.value[id] = true;
  });
});
</script>

<template>
  <v-main>
    <v-select :items="graphOptions" v-model="graphType"></v-select>
    <div v-if="graphType === 'people'">
      <Line :data="peopleTimeline"></Line>
      <div>
        <v-checkbox
          v-for="person in people"
          :key="person.Id"
          :label="person.data.name"
          v-model="enabledPeople[person.Id]"
        ></v-checkbox>
      </div>
    </div>
    <div v-if="graphType === 'photographers'">
      <Line :data="photographerTimeline"></Line>
      <div>
        <v-checkbox
          v-for="person in people"
          :key="person.Id"
          :label="person.data.name"
          v-model="enabledPeople[person.Id]"
        ></v-checkbox>
      </div>
    </div>
    <div v-if="graphType === 'places'">
      <Line :data="placesTimeline"></Line>
      <div>
        <v-checkbox
          v-for="place in places"
          :key="place.Id"
          :label="place.data.name"
          v-model="enabledPlaces[place.Id]"
        ></v-checkbox>
      </div>
    </div>
    <div v-if="graphType === 'tags'">
      <Line :data="tagsTimeline"></Line>
    </div>
    <div v-if="graphType === 'moods'">
      <Scatter :data="moodTimeline"></Scatter>
    </div>
  </v-main>
</template>
