<script setup lang="ts">
import moment from 'moment';
import { ref } from 'vue';
import { Photo } from '../../classes/Photo';
import { fileStore, formatDate } from '../../stores/fileStore';

const route = useRoute();
const router = useRouter();

const { filteredPhotos, filters, setFilter, places, checkFilter, people } = fileStore;

const selected = ref<Photo[]>([]);
const photos = ref<Photo[]>([]);
const filterByLocation = ref(false);
const filterByDate = ref(false);
const filterByPerson = ref(false);
const currentDate = ref(new Date());

fileStore.on('updateFilters', () => {
  photos.value = filteredPhotos(filterByLocation.value, filterByDate.value);
});

fileStore.on('updatePhoto', (photo) => {
  const idx = photos.value.findIndex((p) => p.data.name === photo.data.name);
  if (checkFilter(photo, filterByLocation.value, filterByDate.value, filterByPerson.value)) {
    if (idx < 0) {
      photos.value.push(photo);
    }
  } else {
    if (idx >= 0) {
      photos.value.splice(idx, 1);
    }
  }
  // photos.value = filteredPhotos(filterByLocation.value, filterByDate.value);
});

onMounted(() => {
  if (route.query.place) {
    setFilter('filterPos', route.query.place as string);
    filterByLocation.value = true;
  }
  if (route.query.date) {
    const d = formatDate(new Date(route.query.date as string));
    setFilter('filterDate', d);
    filterByDate.value = true;
    currentDate.value = new Date(Date.parse(filters.filterDate));
  }
  if (route.query.person) {
    setFilter('filterPerson', route.query.person as string);
    filterByPerson.value = true;
  }
  photos.value = filteredPhotos(filterByLocation.value, filterByDate.value, filterByPerson.value);
});
</script>

<template>
  <v-main class="main">
    <v-container fluid>
      <v-row>
        <v-col cols="6" ref="gridCol">
          <div class="flex">
            <tag-input
              label="Tags to include"
              :value="filters.enabledTags"
              @update="
                (tags) => {
                  setFilter('enabledTags', tags);
                }
              "
            ></tag-input>
            <div v-if="filterByLocation">
              <v-btn
                icon
                flat
                @click="
                  () => {
                    filterByLocation = false;
                    photos = filteredPhotos(filterByLocation, filterByDate, filterByPerson);
                  }
                "
              >
                <v-icon>mdi-close</v-icon>
              </v-btn>
              {{ places[route.query.place as string].data.name }}
            </div>
            <div v-if="filterByPerson">
              <v-btn
                icon
                flat
                @click="
                  () => {
                    filterByPerson = false;
                    photos = filteredPhotos(filterByLocation, filterByDate, filterByPerson);
                  }
                "
              >
                <v-icon>mdi-close</v-icon>
              </v-btn>
              {{ people[route.query.person as string].data.name }}
            </div>
            <div v-if="filterByDate">
              <v-btn
                icon
                flat
                @click="
                  () => {
                    filterByDate = false;
                    photos = filteredPhotos(filterByLocation, filterByDate, filterByPerson);
                  }
                "
              >
                <v-icon>mdi-close</v-icon>
              </v-btn>
              <v-btn
                icon
                flat
                @click="
                  () => {
                    currentDate = moment(currentDate).subtract(1, 'day').toDate();
                    setFilter('filterDate', formatDate(currentDate));
                  }
                "
              >
                <v-icon>mdi-arrow-left</v-icon>
              </v-btn>
              {{ formatDate(currentDate) }}
              <v-btn
                icon
                flat
                @click="
                  () => {
                    currentDate = moment(currentDate).add(1, 'day').toDate();
                    setFilter('filterDate', formatDate(currentDate));
                  }
                "
              >
                <v-icon>mdi-arrow-right</v-icon>
              </v-btn>
              <v-btn
                @click="
                  () => {
                    router.push(`/journal?date=${currentDate.toISOString()}`);
                  }
                "
                >Open in Journal</v-btn
              >
            </div>
          </div>
          <photo-grid :photos="photos" @select="(s) => (selected = s)"></photo-grid>
        </v-col>
        <v-col cols="6">
          <v-btn :color="selected.length > 0 ? 'primary' : 'default'" flat @click="selected = []"
            >Clear Selection ({{ selected.length }})</v-btn
          >
          <photo-group v-if="selected.length > 0" :photos="selected"></photo-group>
        </v-col>
      </v-row>
    </v-container>
  </v-main>
</template>

<style scoped></style>
