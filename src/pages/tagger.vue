<script setup lang="ts">
import moment from 'moment';
import { ref, computed, onMounted } from 'vue';
import { useRoute } from 'vue-router';
import { Photo } from '../classes/Photo';
import { fileStore, formatDate } from '../stores/fileStore';
import AutosaveText from '../components/AutosaveText.vue';
import DirectoryPanels from '../components/DirectoryPanels.vue';
import MoodIcon from '../components/MoodIcon.vue';
import PhotoGrid from '../components/PhotoGrid.vue';
import PhotoGroup from '../components/PhotoGroup.vue';
import SearchInput from '../components/SearchInput.vue';

const route = useRoute();

const {
  checkFilter,
  setEntryMood,
  setEntryText,
  journals,
  folder,
  workingDir,
  viewMode,
  setViewMode,
  search,
  query
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
const localQuery = ref<string[]>([]);

// Journal editor
const mood = ref(2);
const entryText = ref('');

async function setDate(date: Date) {
  currentDate.value = date;
  const d = currentDate.value.toISOString();
  await search(`date=${formatDate(currentDate.value)}`);
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

fileStore.on('search', (results) => {
  photos.value = results;
});

fileStore.on('updatePhoto', (photo) => {
  const idx = photos.value.findIndex((p) => p.data.name === photo.data.name);
  if (checkFilter(photo)) {
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

onMounted(async () => {
  if (route.query.place) {
    await search(`at=${route.query.place}`);
    filterByLocation.value = true;
  } else if (route.query.date) {
    setDate(moment(route.query.date as string).toDate());
    filterByDate.value = true;
  } else if (route.query.person) {
    await search(`of=${route.query.person}`);
    filterByPerson.value = true;
  } else if (route.query.photographer) {
    await search(`by=${route.query.photographer}`);
    filterByPhotographer.value = true;
  } else {
    await search(...query);
  }
  localQuery.value = query;
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
            <search-input></search-input>
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
          </div>
          {{ localQuery }}
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
                  await setEntryMood(formatDate(currentDate), newMood);
                  mood = newMood;
                }
              "
            ></mood-icon>
            <autosave-text
              :value="entryText"
              @save="
                async (text) => {
                  await setEntryText(formatDate(currentDate), text);
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
