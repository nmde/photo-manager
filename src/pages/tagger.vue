<script setup lang="ts">
  import type { Photo } from '@/classes/Photo';
  import { photo_grid, refresh, type Sort } from '@/api/app';
  import { useFileStore } from '@/stores/fileStore';

  const route = useRoute();

  const store = useFileStore();
  const { reportError } = store;
  const { query, sortBy, itemsPerRow } = storeToRefs(store);

  const selected = ref<Photo[]>([]);
  const photos = ref<Photo[]>([]);
  const searching = ref(false);
  const sorting = ref(false);
  const current = ref(0);
  const showDetail = ref(false);
  const lastSelectedIndex = ref(0);
  const inputFocus = ref(false);
  const refreshing = ref(false);

  async function searchGrid() {
    searching.value = true;
    await photo_grid(query.value, sortBy.value)
      .ok(p => (photos.value = p))
      .err(reportError)
      .send();
    searching.value = false;
  }

  async function setSortMode(sort: Sort) {
    sorting.value = true;
    store.setQuery(query.value, sort);
    await searchGrid();
    sorting.value = false;
  }

  /**
   * Handles selecting one or more photos.
   * @param photo - The photo being selected.
   * @param index - The index of the photo selected.
   */
  function selectPhoto(photo: Photo, index: number) {
    const s: Photo[] = [photo];
    if (shiftPressed.value && index !== lastSelectedIndex.value) {
      const startIndex = Math.min(index, lastSelectedIndex.value);
      const endIndex = Math.max(index, lastSelectedIndex.value);
      const range: Photo[] = [];
      for (let i = startIndex; i <= endIndex; i += 1) {
        const p = photos.value[i] as Photo | undefined;
        if (p) {
          range.push(p);
        }
      }
      selected.value = range;
      if (current.value >= selected.value.length) {
        current.value = selected.value.length - 1;
      }
    } else if (ctrlPressed.value) {
      for (const x of s) {
        const idx = selected.value.findIndex(p => p.name === x.name);
        if (idx === -1) {
          selected.value.push(x);
        } else {
          selected.value.splice(idx, 1);
        }
      }
      if (current.value >= selected.value.length) {
        current.value = selected.value.length - 1;
      }
    } else {
      selected.value = s;
      current.value = 0;
    }
    lastSelectedIndex.value = index;
  }

  const loading = ref(false);
  const targetTags = ref<string[]>([]);
  const tagAddDialog = ref(false);
  const tagReplaceDialog = ref(false);

  async function addTags() {
    loading.value = true;
    for (const photo of selected.value) {
      const combinedTags = new Set(photo.tags);
      for (const tag of targetTags.value) {
        combinedTags.add(tag);
      }
      await photo.setTags(combinedTags.values().toArray());
    }
    loading.value = false;
    tagAddDialog.value = false;
  }

  const tagAction = ref('replace');
  const replacementTag = ref<string[]>([]);
  async function replaceTags() {
    loading.value = true;
    const tag = targetTags.value[0];
    if (tag) {
      for (const photo of selected.value) {
        if (tagAction.value === 'remove') {
          const updatedTags = [...photo.tags];
          updatedTags.splice(updatedTags.indexOf(tag), 1);
          await photo.setTags(updatedTags);
        } else {
          const updatedTags = [...photo.tags];
          updatedTags.splice(updatedTags.indexOf(tag), 1);
          if (replacementTag.value[0]) {
            updatedTags.push(replacementTag.value[0]);
          }
          await photo.setTags(updatedTags);
        }
      }
    }
    loading.value = false;
    tagReplaceDialog.value = false;
  }

  async function quickGroup(groupName: string) {
    const target = [...selected.value];
    selected.value = [];
    for (const photo of target) {
      await photo.setGroup(groupName);
    }
  }

  async function refreshGrid() {
    refreshing.value = true;
    await refresh();
    await searchGrid();
    refreshing.value = false;
  }

  const gridSection = ref<HTMLDivElement>();
  const photoGridWidth = ref(0);
  const ctrlPressed = ref(false);
  const shiftPressed = ref(false);
  onMounted(async () => {
    if (gridSection.value) {
      photoGridWidth.value = gridSection.value.getBoundingClientRect()?.width;
    }
    if (route.query.date) {
      query.value = ['has:date', `date:${route.query.date}`];
    } else if (route.query.person) {
      query.value = ['has:people', `of:${route.query.person}`];
    } else if (route.query.tag) {
      query.value = ['has:tags', route.query.tag as string];
    }
    window.addEventListener('keydown', event => {
      if (event.ctrlKey) {
        ctrlPressed.value = true;
      }
      if (event.shiftKey) {
        shiftPressed.value = true;
      }
      if (!inputFocus.value) {
        if (event.key === 'a' && event.ctrlKey) {
          event.preventDefault();
          selected.value = photos.value;
        }
        if (event.key === '-' && event.ctrlKey) {
          event.preventDefault();
          store.setItemsPerRow(itemsPerRow.value + 1);
        }
        if (event.key === '=' && event.ctrlKey) {
          event.preventDefault();
          store.setItemsPerRow(itemsPerRow.value - 1);
        }
        if (event.key === 'ArrowRight') {
          if (selected.value.length > 1 && current.value + 1 < selected.value.length) {
            current.value += 1;
          } else if (selected.value.length === 1) {
            const idx = photos.value.findIndex(p => p.name === selected.value[0]?.name);
            const nextPhoto = photos.value[idx + 1];
            if (nextPhoto) {
              selected.value = [nextPhoto];
            }
          } else if (selected.value.length === 0) {
            const firstPhoto = photos.value[0];
            if (firstPhoto) {
              selected.value = [firstPhoto];
            }
          }
          showDetail.value = true;
        }
        if (event.key === 'ArrowLeft') {
          if (selected.value.length > 1 && current.value - 1 >= 0) {
            current.value -= 1;
          } else if (selected.value.length === 1) {
            const idx = photos.value.findIndex(p => p.name === selected.value[0]?.name);
            const prevPhoto = photos.value[idx - 1];
            if (prevPhoto) {
              selected.value = [prevPhoto];
            }
          } else if (selected.value.length === 0) {
            const firstPhoto = photos.value[0];
            if (firstPhoto) {
              selected.value = [firstPhoto];
            }
          }
          showDetail.value = true;
        }
        if (event.key === 'ArrowDown' && selected.value.length === 1) {
          const idx = photos.value.findIndex(p => p.name === selected.value[0]?.name);
          const downPhoto = photos.value[idx + itemsPerRow.value];
          if (downPhoto) {
            selected.value = [downPhoto];
          }
        }
        if (event.key === 'ArrowUp' && selected.value.length === 1) {
          const idx = photos.value.findIndex(p => p.name === selected.value[0]?.name);
          const upPhoto = photos.value[idx - itemsPerRow.value];
          if (upPhoto) {
            selected.value = [upPhoto];
          }
        }
      }
    });
    window.addEventListener('keyup', event => {
      if (ctrlPressed.value && event.key === 'Control') {
        ctrlPressed.value = false;
      }
      if (shiftPressed.value && event.key === 'Shift') {
        shiftPressed.value = false;
      }
    });
    await searchGrid();
  });
</script>

<template>
  <div class="tagger-page">
    <div ref="gridSection" :class="{ 'grid-section': true, 'grid-section--full': !showDetail }">
      <v-toolbar color="primary">
        <v-menu>
          <template #activator="{ props: bprops }">
            <v-btn v-bind="bprops" flat icon :loading="sorting">
              <v-icon>mdi-sort</v-icon>
            </v-btn>
          </template>
          <v-list>
            <v-list-item @click="setSortMode('name')">Sort by name</v-list-item>
            <v-list-item @click="setSortMode('namedesc')"> Sort by name (desc) </v-list-item>
            <v-list-item @click="setSortMode('rating')">Sort by rating</v-list-item>
            <v-list-item @click="setSortMode('ratingdesc')"> Sort by rating (desc) </v-list-item>
            <v-list-item @click="setSortMode('date')">Sort by date</v-list-item>
            <v-list-item @click="setSortMode('datedesc')"> Sort by date (desc) </v-list-item>
          </v-list>
        </v-menu>
        <div class="toolbar-controls">
          <v-combobox
            v-model="query"
            chips
            clearable
            density="compact"
            label="Search"
            multiple
            variant="outlined"
            @update:focused="val => (inputFocus = val)"
          >
            <template #append>
              <v-btn density="compact" icon :loading="searching" @click="searchGrid()">
                <v-icon>mdi-magnify</v-icon>
              </v-btn>
              <v-btn density="compact" icon :loading="refreshing" @click="refreshGrid()">
                <v-icon>mdi-refresh</v-icon>
              </v-btn>
            </template>
          </v-combobox>
          <template v-if="selected.length > 1">
            <v-menu>
              <template #activator="{ props }">
                <v-btn v-bind="props" icon>
                  <v-icon style="top: -10px">mdi-dots-vertical</v-icon>
                </v-btn>
              </template>
              <v-list>
                <v-list-item @click="tagAddDialog = true">Add Tag to Selected</v-list-item>
                <v-list-item @click="tagReplaceDialog = true">
                  Replace/Remove Tag From Selected
                </v-list-item>
                <v-list-item @click="quickGroup(selected[0]?.name ?? 'NewGroup')">
                  Group Selected
                </v-list-item>
              </v-list>
            </v-menu>
          </template>
        </div>
        <template #append>
          <v-btn flat icon @click="store.setItemsPerRow(itemsPerRow + 1)">
            <v-icon>mdi-minus</v-icon>
          </v-btn>
          <v-btn flat icon @click="store.setItemsPerRow(itemsPerRow - 1)">
            <v-icon>mdi-plus</v-icon>
          </v-btn>
        </template>
      </v-toolbar>
      <photo-grid
        :half-width="showDetail"
        :items-per-row="itemsPerRow"
        :photos="photos as Photo[]"
        :selected="selected as Photo[]"
        @select="
          (photo, index) => {
            selectPhoto(photo, index);
            showDetail = true;
          }
        "
      />
    </div>
    <div :class="{ 'photo-pane': true, 'photo-pane--visible': showDetail }">
      <v-toolbar color="secondary">
        <template v-if="selected.length > 1">
          <v-btn
            flat
            icon
            @click="
              () => {
                if (current > 0) {
                  current -= 1;
                }
              }
            "
          >
            <v-icon>mdi-arrow-left</v-icon>
          </v-btn>
          {{ current + 1 }} / {{ selected.length }}
          <v-btn
            flat
            icon
            @click="
              () => {
                if (current < selected.length - 1) {
                  current += 1;
                }
              }
            "
          >
            <v-icon>mdi-arrow-right</v-icon>
          </v-btn>
        </template>
        <v-btn v-else icon @click="showDetail = false">
          <v-icon>mdi-arrow-collapse-right</v-icon>
        </v-btn>
        <v-toolbar-title class="photo-name">{{ selected[current]?.name }}</v-toolbar-title>
        <v-spacer />
        <v-btn
          v-if="selected.length > 0"
          variant="tonal"
          @click="
            {
              selected = [];
              showDetail = false;
            }
          "
        >
          Clear Selection ({{ selected.length }})
        </v-btn>
      </v-toolbar>
      <photo-detail
        :index="current"
        :photos="selected as Photo[]"
        @input-focused="val => (inputFocus = val)"
      />
    </div>
  </div>
  <v-dialog v-model="tagAddDialog" max-width="700">
    <v-card title="Add Tags">
      <v-card-text>
        Add a tag to selected photos (<b>this action will effect {{ selected.length }} photos</b>!)
        <tag-input label="Tag to add" :value="targetTags" @change="tags => (targetTags = tags)" />
        <v-card-actions>
          <v-spacer />
          <v-btn @click="tagAddDialog = false">Cancel</v-btn>
          <v-btn color="primary" :loading="loading" @click="addTags()"> Apply Changes </v-btn>
        </v-card-actions>
      </v-card-text>
    </v-card>
  </v-dialog>
  <v-dialog v-model="tagReplaceDialog" max-width="700">
    <v-card title="Remove or Replace">
      <v-card-title>Remove and Replace Tags</v-card-title>
      <v-card-text>
        Search for a tag to remove (<b>this action will effect {{ selected.length }} photos</b>!)
        <tag-input label="Tag to find" :value="targetTags" @change="tag => (targetTags = tag)" />
        <v-radio-group v-model="tagAction">
          <v-radio label="Remove tag" value="remove" />
          <v-radio label="Replace tag" value="replace" />
        </v-radio-group>
        <div v-if="tagAction === 'replace'">
          Replace with:
          <tag-input
            label="Tag to replace with"
            :value="replacementTag"
            @change="tag => (replacementTag = tag)"
          />
          Replacing {{ targetTags[0] }} with {{ replacementTag[0] }}.
        </div>
      </v-card-text>
      <v-card-actions>
        <v-spacer />
        <v-btn color="primary" :loading="loading" @click="replaceTags()"> Apply Changes </v-btn>
        <v-btn @click="tagReplaceDialog = false">Cancel</v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<style scoped>
  .tagger-page {
    display: flex;
  }

  .tagger-page > div {
    transition: width 150ms ease-in;
  }

  .photo-pane {
    width: 0;
  }

  .grid-section,
  .photo-pane--visible {
    width: 50%;
  }

  .grid-section--full {
    width: 100%;
  }
</style>

<style>
  .photo-name > .v-toolbar-title__placeholder {
    overflow: visible;
  }
</style>
