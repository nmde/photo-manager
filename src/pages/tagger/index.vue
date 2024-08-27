<script setup lang="ts">
import moment from 'moment';
import { ref } from 'vue';
import { Photo } from '../../classes/Photo';
import { fileStore, formatDate } from '../../stores/fileStore';

const route = useRoute();
const router = useRouter();

const {
  filteredPhotos,
  filters,
  setFilter,
  places,
  checkFilter,
  people,
  files,
  setEntryMood,
  setEntryText,
  journals,
} = fileStore;

const selected = ref<Photo[]>([]);
const photos = ref<Photo[]>([]);
const filterByLocation = ref(false);
const filterByDate = ref(false);
const filterByPerson = ref(false);
const currentDate = ref(new Date());

// Journal editor
const mood = ref(2);
const entryText = ref('');

function setDate(date: Date) {
  currentDate.value = date;
  const d = currentDate.value.toISOString();
  setFilter('filterDate', formatDate(currentDate.value));
  if (journals[d]) {
    mood.value = journals[d].data.mood;
    entryText.value = journals[d].data.text;
  } else {
    mood.value = 2;
    entryText.value = '';
  }
}

fileStore.on('updateFilters', () => {
  console.log('Updating filters');
  if (
    !filterByLocation.value &&
    !filterByDate.value &&
    !filterByPerson.value &&
    filters.enabledTags.length === 0 &&
    filters.disabledTags.length == 0 &&
    !filters.hideTagged &&
    !filters.hideLocated
  ) {
    console.log('Filters all disabled');
    photos.value = Object.values(files);
  } else {
    photos.value = filteredPhotos(filterByLocation.value, filterByDate.value, filterByPerson.value);
  }
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
    setDate(moment(route.query.date as string).toDate());
    filterByDate.value = true;
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
                @click="() => setDate(moment(currentDate).subtract(1, 'day').toDate())"
              >
                <v-icon>mdi-arrow-left</v-icon>
              </v-btn>
              {{ formatDate(currentDate) }}
              <v-btn icon flat @click="() => setDate(moment(currentDate).add(1, 'day').toDate())">
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
          <div v-if="filterByDate">
            <mood-icon
              :mood="mood"
              @selected="
                async (newMood) => {
                  await setEntryMood(currentDate.toISOString(), newMood);
                  mood = newMood;
                }
              "
            ></mood-icon>
            <autosave-text
              :value="entryText"
              @save="
                async (text) => {
                  await setEntryText(currentDate.toISOString(), text);
                  entryText = text;
                }
              "
            ></autosave-text>
          </div>
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
