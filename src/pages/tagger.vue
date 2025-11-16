<script setup lang="ts">
  import type { Photo } from '../classes/Photo';
  import moment from 'moment';
  import { computed, onMounted, ref } from 'vue';
  import { useRoute } from 'vue-router';
  import PhotoGrid from '@/components/PhotoGrid.vue';
  import { fileStore, formatDate } from '../stores/fileStore';

  const route = useRoute();

  const { setEntryText, journals, viewMode, setViewMode, query } = fileStore;

  const grid = ref<InstanceType<typeof PhotoGrid>>();

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
  const searching = ref(false);

  // Journal editor
  const mood = ref(2);
  const entryText = ref('');

  async function searchGrid(query: string[]) {
    searching.value = true;
    await grid.value?.search(query);
    searching.value = false;
  }

  async function setDate(date: Date) {
    currentDate.value = date;
    const d = currentDate.value.toISOString();
    await searchGrid([`date=${d}`]);
    if (journals[d]) {
      mood.value = journals[d].mood;
      entryText.value = journals[d].displayText;
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
    /* TODO
    for (const dir of folder.dirs) {
      const split = dir.replace(workingDir, '').split(/[/\\]/).slice(1);
      let curr = structure;
      for (const seg of split) {
        if (!curr.children[seg]) {
          curr.children[seg] = {
            files: [],
            children: {},
          };
        }
        curr = curr.children[seg];
      }
    }
    for (const file of photos.value.map(p => p.name)) {
      const split = file.replace(workingDir, '').split(/[/\\]/).slice(1);
      let curr = structure;
      for (let i = 0; i <= split.length - 2; i += 1) {
        const s = split[i];
        // I store my photos in dropbox, and this condition catches an issue with dropbox folders.
        // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition
        if (curr === undefined) {
          console.warn(`Undefined value in folder structure ${split.join(',')}`);
        } else if (s && curr.children[s]) {
          curr = curr.children[s];
        }
      }
      curr.files.push(file);
    }
      */
    return structure;
  });

  function syncRating() {
    grid.value?.updateRating();
  }

  onMounted(async () => {
    if (route.query.place) {
      await searchGrid([`at:${route.query.place as string}`]);
      filterByLocation.value = true;
    } else if (route.query.date) {
      await setDate(moment(route.query.date as string).toDate());
      filterByDate.value = true;
    } else if (route.query.person) {
      await searchGrid([`of:${route.query.person as string}`]);
      filterByPerson.value = true;
    } else if (route.query.photographer) {
      await searchGrid([`by:${route.query.photographer as string}`]);
      filterByPhotographer.value = true;
    }
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
        <v-col ref="gridCol" cols="6">
          <div class="flex">
            <search-input
              :loading="searching"
              :value="query"
              @search="async query => searchGrid(query)"
            />
            <v-btn
              v-if="localViewMode === 0"
              @click="
                localViewMode = 1;
                setViewMode(1);
              "
            >
              View Folders
            </v-btn>
            <v-btn
              v-if="localViewMode === 1"
              @click="
                localViewMode = 0;
                setViewMode(0);
              "
            >
              View Grid
            </v-btn>
          </div>
          <photo-grid
            v-if="localViewMode === 0"
            ref="grid"
            :photos="photos as Photo[]"
            @select="s => (selected = s)"
          />
          <div v-if="localViewMode === 1">
            <directory-panels :folder-structure="folderStructure" @select="s => (selected = s)" />
          </div>
          <div v-if="filterByDate">
            <mood-icon
              :mood="mood"
              @selected="
                async newMood => {
                  await journals[formatDate(currentDate)]?.setMood(newMood);
                  mood = newMood;
                }
              "
            />
            <autosave-text
              :value="entryText"
              @save="
                async text => {
                  await setEntryText(formatDate(currentDate), text);
                  entryText = text;
                }
              "
            />
          </div>
        </v-col>
        <v-col cols="6">
          <div class="details">
            <v-btn :color="selected.length > 0 ? 'primary' : 'default'" flat @click="selected = []">
              Clear Selection ({{ selected.length }})
            </v-btn>
            <photo-group
              v-if="selected.length > 0"
              :photos="selected as Photo[]"
              :prev-date="prevDate"
              @update-date="date => (prevDate = date)"
              @update-rating="syncRating"
            />
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
    margin-top: 8px;
    overflow: scroll;
    top: 6px;
    width: -webkit-fill-available;
  }
</style>
