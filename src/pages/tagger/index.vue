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
  folder,
  workingDir,
  viewMode,
  setViewMode,
} = fileStore;

const selected = ref<Photo[]>([]);
const photos = ref<Photo[]>([]);
const filterByLocation = ref(false);
const filterByDate = ref(false);
const filterByPerson = ref(false);
const filterByPhotographer = ref(false);
const currentDate = ref(new Date());
const localViewMode = ref(0);
const spacer = ref(false);
const prevDate = ref<Date>(new Date());

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

type Folder = {
  files: string[];
  children: Record<string, Folder>;
};

// Folder view
const folderStructure = computed(() => {
  const structure: Folder = {
    files: [],
    children: {},
  };
  folder.dirs.forEach((dir) => {
    const split = dir.replace(workingDir, '').split(/[/\\]/).slice(1);
    let curr = structure;
    split.forEach((seg) => {
      if (!curr.children[seg]) {
        curr.children[seg] = {
          files: [],
          children: {},
        };
      }
      curr = curr.children[seg];
    });
  });
  photos.value
    .map((p) => p.data.name)
    .forEach((file) => {
      const split = file.replace(workingDir, '').split(/[/\\]/).slice(1);
      let curr = structure;
      for (let i = 0; i <= split.length - 2; i += 1) {
        curr = curr.children[split[i]];
      }
      curr.files.push(file);
    });
  return structure;
});

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
    photos.value = filteredPhotos(
      filterByLocation.value,
      filterByDate.value,
      filterByPerson.value,
      filterByPhotographer.value,
    );
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
  if (route.query.photographer) {
    setFilter('filterPhotographer', route.query.photographer as string);
    filterByPhotographer.value = true;
  }
  photos.value = filteredPhotos(
    filterByLocation.value,
    filterByDate.value,
    filterByPerson.value,
    filterByPhotographer.value,
  );
  localViewMode.value = viewMode;
});

window.addEventListener('scroll', () => {
  spacer.value = window.scrollY < 100;
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
            <v-btn
              v-if="localViewMode === 0"
              @click="
                localViewMode = 1;
                setViewMode(1);
              "
              >View Folders</v-btn
            >
            <v-btn
              v-if="localViewMode === 1"
              @click="
                localViewMode = 0;
                setViewMode(0);
              "
              >View Grid</v-btn
            >
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
            <div v-if="filterByPhotographer">
              <v-btn
                icon
                flat
                @click="
                  () => {
                    filterByPhotographer = false;
                    photos = filteredPhotos(
                      filterByLocation,
                      filterByDate,
                      filterByPerson,
                      filterByPhotographer,
                    );
                  }
                "
              >
                <v-icon>mdi-close</v-icon>
              </v-btn>
              {{ people[route.query.photographer as string].data.name }}
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
          <photo-grid
            v-if="localViewMode === 0"
            :photos="photos"
            @select="(s) => (selected = s)"
          ></photo-grid>
          <div v-if="localViewMode === 1">
            <directory-panels
              :folder-structure="folderStructure"
              @select="(s) => (selected = s)"
            ></directory-panels>
          </div>
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
          <div class="details" :class="{ spacer: spacer }">
            <v-btn :color="selected.length > 0 ? 'primary' : 'default'" flat @click="selected = []"
              >Clear Selection ({{ selected.length }})</v-btn
            >
            <photo-group
              :prev-date="prevDate"
              v-if="selected.length > 0"
              :photos="selected"
              @update-date="
                (date) => {
                  prevDate = date;
                }
              "
            ></photo-group>
          </div>
        </v-col>
      </v-row>
    </v-container>
  </v-main>
</template>

<style scoped>
.details {
  position: fixed;
  height: 100%;
  overflow: scroll;
  top: 6px;
  width: -webkit-fill-available;
}

.details.spacer {
  top: 80px;
}
</style>
