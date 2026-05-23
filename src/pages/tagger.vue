<script setup lang="ts">
  import type { Photo } from '@/classes/Photo';
  import { useRules } from 'vuetify/labs/rules';
  import { photo_grid, refresh, type Sort } from '@/api/app';
  import { get_group } from '@/api/photos';
  import { useFileStore } from '@/stores/fileStore';

  const route = useRoute();
  const rules = useRules();

  const store = useFileStore();
  const { reportError } = store;
  const { query, sortBy, itemsPerRow } = storeToRefs(store);

  const selected = shallowRef<Photo[]>([]);
  const photos = shallowRef<Photo[]>([]);
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
   * Expands the photo selection to include photos within groups
   * @param photos - The top-level selected photos.
   */
  async function getPhotosForGroup(photos: Photo[]) {
    const visited: string[] = [];
    let re: Photo[] = [];
    for (const photo of photos) {
      if (photo.group === null) {
        re.push(photo);
      } else if (!visited.includes(photo.group)) {
        visited.push(photo.group);
        await get_group(photo.group)
          .ok(photos => {
            re = re.concat(photos);
          })
          .err(reportError)
          .send();
      }
    }
    return re;
  }

  /**
   * Handles selecting one or more photos.
   * @param photo - The photo being selected.
   * @param index - The index of the photo selected.
   */
  async function selectPhoto(photo: Photo, index: number) {
    const s = [photo];
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
      selected.value = await getPhotosForGroup(range);
      if (current.value >= selected.value.length) {
        current.value = selected.value.length - 1;
      }
    } else if (ctrlPressed.value) {
      const initial = [...selected.value];
      for (const x of s) {
        const idx = selected.value.findIndex(p => p.name === x.name);
        if (idx === -1) {
          initial.push(x);
        } else {
          initial.splice(idx, 1);
        }
      }
      selected.value = await getPhotosForGroup(initial);
      if (current.value >= selected.value.length) {
        current.value = selected.value.length - 1;
      }
    } else {
      selected.value = await getPhotosForGroup(s);
      current.value = 0;
    }
    lastSelectedIndex.value = index;
  }

  const tagAddDialog = ref(false);
  const tagReplaceDialog = ref(false);

  type ReplaceTagFields = {
    target?: string[];
    action: 'remove' | 'replace';
    replacement?: string[];
  };
  const replaceTagFields = ref<ReplaceTagFields>({ action: 'remove' });

  async function addTags() {
    const fields = replaceTagFields.value as Required<ReplaceTagFields>;
    for (const photo of selected.value) {
      const combinedTags = new Set(photo.tags);
      for (const tag of fields.target) {
        combinedTags.add(tag);
      }
      await photo.setTags(combinedTags.values().toArray());
    }
  }

  async function replaceTags() {
    const fields = replaceTagFields.value as Required<ReplaceTagFields>;
    const tag = fields.target[0];
    if (tag) {
      for (const photo of selected.value) {
        if (fields.action === 'remove') {
          const updatedTags = [...photo.tags];
          updatedTags.splice(updatedTags.indexOf(tag), 1);
          await photo.setTags(updatedTags);
        } else {
          const updatedTags = [...photo.tags];
          updatedTags.splice(updatedTags.indexOf(tag), 1);
          if (fields.replacement[0]) {
            updatedTags.push(fields.replacement[0]);
          }
          await photo.setTags(updatedTags);
        }
      }
    }
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

  const gridSection = useTemplateRef('gridSection');
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
        // Crtl a: Select all
        if (event.key === 'a' && event.ctrlKey) {
          event.preventDefault();
          selected.value = photos.value;
        }
        // Crtl -: Zoom grid out
        if (event.key === '-' && event.ctrlKey) {
          event.preventDefault();
          store.setItemsPerRow(itemsPerRow.value + 1);
        }
        // Ctrl +: Zoom grid in
        if (event.key === '=' && event.ctrlKey) {
          event.preventDefault();
          store.setItemsPerRow(itemsPerRow.value - 1);
        }
        if (event.key === 'ArrowRight') {
          // Shift ->: Extend selection to the right
          if (shiftPressed.value && selected.value.length > 0) {
            const nextPhoto = photos.value[lastSelectedIndex.value + 1];
            if (nextPhoto !== undefined) {
              selected.value = [...selected.value, nextPhoto];
              lastSelectedIndex.value += 1;
              current.value = selected.value.length - 1;
            }
            // ->: Move right between selected photos in the details pane
          } else if (selected.value.length > 1 && current.value + 1 < selected.value.length) {
            current.value += 1;
            // ->: Move to photo to the right
          } else if (selected.value.length === 1) {
            const idx = photos.value.findIndex(p => p.name === selected.value[0]?.name);
            const nextPhoto = photos.value[idx + 1];
            if (nextPhoto !== undefined) {
              selected.value = [nextPhoto];
              lastSelectedIndex.value = idx + 1;
            }
            // ->: Select first photo in the grid
          } else if (selected.value.length === 0) {
            const firstPhoto = photos.value[0];
            if (firstPhoto) {
              selected.value = [firstPhoto];
              lastSelectedIndex.value = 0;
            }
          }
          showDetail.value = true;
        }
        if (event.key === 'ArrowLeft') {
          // Shift <-: Remove last selected
          if (shiftPressed.value && selected.value.length > 0) {
            // eslint-disable-next-line unicorn/no-useless-spread
            selected.value = [...selected.value.slice(0, -1)];
            lastSelectedIndex.value -= 1;
            current.value -= 1;
            // <-: Move left between selected photos in the details pane
          } else if (selected.value.length > 1 && current.value - 1 >= 0) {
            current.value -= 1;
            // <-: Move to photo to the left
          } else if (selected.value.length === 1) {
            const idx = photos.value.findIndex(p => p.name === selected.value[0]?.name);
            const prevPhoto = photos.value[idx - 1];
            if (prevPhoto) {
              selected.value = [prevPhoto];
              lastSelectedIndex.value = idx - 1;
            }
            // <-: Select last photo in the grid
          } else if (selected.value.length === 0) {
            const lastPhoto = photos.value.at(-1);
            if (lastPhoto !== undefined) {
              selected.value = [lastPhoto];
              lastSelectedIndex.value = photos.value.length - 1;
            }
          }
          showDetail.value = true;
        }
        if (event.key === 'ArrowDown' && selected.value.length === 1) {
          const idx = photos.value.findIndex(p => p.name === selected.value[0]?.name);
          const downPhoto = photos.value[idx + itemsPerRow.value];
          if (downPhoto !== undefined) {
            selected.value = [downPhoto];
            lastSelectedIndex.value = idx + itemsPerRow.value;
          }
        }
        if (event.key === 'ArrowUp' && selected.value.length === 1) {
          const idx = photos.value.findIndex(p => p.name === selected.value[0]?.name);
          const upPhoto = photos.value[idx - itemsPerRow.value];
          if (upPhoto !== undefined) {
            selected.value = [upPhoto];
            lastSelectedIndex.value = idx - itemsPerRow.value;
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
            <v-list-item @click="setSortMode('name_desc')"> Sort by name (desc) </v-list-item>
            <v-list-item @click="setSortMode('rating')">Sort by rating</v-list-item>
            <v-list-item @click="setSortMode('rating_desc')"> Sort by rating (desc) </v-list-item>
            <v-list-item @click="setSortMode('date')">Sort by date</v-list-item>
            <v-list-item @click="setSortMode('date_desc')"> Sort by date (desc) </v-list-item>
            <v-list-item @click="setSortMode('filedate')">Sort by file date</v-list-item>
            <v-list-item @click="setSortMode('filedate_desc')">
              Sort by file date (desc)
            </v-list-item>
          </v-list>
        </v-menu>
        <div class="toolbar-controls">
          <SearchBar
            v-model="query"
            v-model:focused="inputFocus"
            v-model:refreshing="refreshing"
            v-model:searching="searching"
            @refresh="refreshGrid()"
            @search="searchGrid()"
          />
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
        :loading="searching || refreshing"
        :photos="photos"
        :selected="selected"
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
        <v-toolbar-title class="photo-name">&nbsp;{{ selected[current]?.name }}</v-toolbar-title>
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
        :photos="selected"
        @input-focused="val => (inputFocus = val)"
      />
    </div>
  </div>
  <form-dialog
    v-model="tagAddDialog"
    :reset="() => (replaceTagFields = { action: 'remove' })"
    save-text="Apply Changes"
    title="Add Tags"
    @submit="async () => await addTags()"
  >
    Add a tag to selected photos (<b>this action will effect {{ selected.length }} photos</b>!)
    <tag-input
      label="Tag to add"
      :rules="[rules.required('A tag to add is required.')]"
      :value="replaceTagFields.target ?? []"
      @change="tags => (replaceTagFields.target = tags)"
    />
  </form-dialog>
  <form-dialog
    v-model="tagReplaceDialog"
    :reset="() => (replaceTagFields = { action: 'remove' })"
    save-text="Apply Changes"
    title="Remove or Replace Tags"
    @submit="async () => await replaceTags()"
  >
    Search for a tag to remove (this action will effect <b>{{ selected.length }} photos</b>!)
    <tag-input
      label="Tag to find"
      :rules="[rules.required('A target tag is required.')]"
      :value="replaceTagFields.target ?? []"
      @change="tag => (replaceTagFields.target = tag)"
    />
    <v-radio-group v-model="replaceTagFields.action">
      <v-radio label="Remove tag" value="remove" />
      <v-radio label="Replace tag" value="replace" />
    </v-radio-group>
    <div v-if="replaceTagFields.action === 'replace'">
      Replace with:
      <tag-input
        label="Tag to replace with"
        :rules="[rules.required('A replacement tag is required.')]"
        :value="replaceTagFields.replacement ?? []"
        @change="tag => (replaceTagFields.replacement = tag)"
      />
      Will replace {{ replaceTagFields.target?.[0] ?? '...' }} with
      {{ replaceTagFields.replacement?.[0] ?? '...' }}.
    </div>
  </form-dialog>
</template>

<style scoped>
  .tagger-page {
    display: flex;
  }

  .tagger-page > div {
    transition: width var(--duration-standard) var(--ease-standard);
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
  .photo-name {
    flex: 1;
    min-width: 0;
  }

  .photo-name > .v-toolbar-title__placeholder {
    overflow: hidden;
    white-space: nowrap;
    text-overflow: ellipsis;
    direction: rtl;
    text-align: left;
  }
</style>
