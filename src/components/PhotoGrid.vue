<script setup lang="ts">
import { computed, ref } from 'vue';
import { Photo } from '../classes/Photo';
import { fileStore } from '../stores/fileStore';

const props = defineProps<{
  photos: Photo[];
}>();

const emit = defineEmits<{
  (e: 'select', photos: Photo[]): void;
}>();

const { getByGroup, addGroup, setGroup, photoCount, filters, setFilter, updateTagsForGroup } =
  fileStore;

const selectMultiple = ref(false);
const selected = ref<Photo[]>([]);
const searchDialog = ref(false);
const itemsPerRow = ref(4);
const size = ref(0);
const gridCol = ref<any>();
const sortBy = ref(0);
const sortDir = ref(1);

/**
 * Resizes the grid items when the window size changes
 */
function resizeGrid() {
  size.value = gridCol.value?.$el.getBoundingClientRect().width / itemsPerRow.value - 8;
}

const rows = computed(() => {
  return Math.min(8, Math.ceil(props.photos.length % size.value));
});

type GridRow = Photo[];

const forceRefresh = ref(false);

// Filters the photos based on the options
const filteredPhotos = computed(() => {
  let f = forceRefresh.value;
  const rows: GridRow[] = [];
  let row: GridRow = [];
  const tempphotos = [...props.photos].sort((a, b) => {
    if (sortBy.value === 1) {
      if (!b.rating) {
        return 1 * sortDir.value;
      }
      if (!a.rating) {
        return -1 * sortDir.value;
      }
      if (a.rating > b.rating) {
        return 1 * sortDir.value;
      }
      if (a.rating < b.rating) {
        return -1 * sortDir.value;
      }
      return 0;
    } else if (sortBy.value === 2) {
      if (b.data.date.length === 0) {
        return -1 * sortDir.value;
      }
      if (a.date > b.date) {
        return 1 * sortDir.value;
      }
      if (a.date < b.date) {
        return -1 * sortDir.value;
      }
      return 0;
    } else {
      if (a.data.name > b.data.name) {
        return 1 * sortDir.value;
      }
      if (a.data.name < b.data.name) {
        return -1 * sortDir.value;
      }
      return 0;
    }
  });
  const groups: string[] = [];
  while (tempphotos.length > 0) {
    const file = tempphotos[0];
    let grouped = false;
    if (typeof file.group === 'string') {
      if (groups.indexOf(file.group) >= 0) {
        grouped = true;
      } else {
        groups.push(file.group);
      }
    }
    if (!grouped) {
      row.push(file);
      if (row.length === itemsPerRow.value) {
        rows.push(row);
        row = [];
      }
    }
    tempphotos.shift();
  }
  rows.push(row);
  return rows;
});

fileStore.on('updatePhoto', () => {
  forceRefresh.value = !forceRefresh.value;
});

// The number of visible photos after filter rules are applied
const visiblePhotoCount = computed(() => {
  return (
    (filteredPhotos.value.length - 1) * itemsPerRow.value +
    filteredPhotos.value[filteredPhotos.value.length - 1].length
  );
});

/**
 * Handles selecting one or more photos.
 * @param photo - The photo being selected.
 */
function selectPhoto(photo: Photo) {
  let s: Photo[] = [photo];
  if (photo.group) {
    s = getByGroup(photo.group);
  }
  if (selectMultiple.value) {
    s.forEach((x) => {
      const idx = selected.value.findIndex((p) => p.data.name === x.data.name);
      if (idx >= 0) {
        selected.value.splice(idx, 1);
      } else {
        selected.value.push(x);
      }
    });
  } else {
    selected.value = s;
  }
  emit('select', selected.value);
}

// Tag add/replace dialogs
const tagReplaceDialog = ref(false);
const tagAddDialog = ref(false);
const targetTag = ref<string[]>([]);
const tagAction = ref<'remove' | 'replace'>('remove');
const replacementTag = ref<string[]>([]);
const loading = ref(false);

// Filter controls
const enabledTags = ref([]);
const disabledTags = ref([]);
const onlyTagged = ref(false);
const hideTagged = ref(false);
const onlyLocated = ref(false);
const hideLocated = ref(false);
const onlyError = ref(false);
const hideDuplicates = ref(true);

onMounted(() => {
  enabledTags.value = fileStore.filters.enabledTags;
  disabledTags.value = fileStore.filters.disabledTags;
  onlyTagged.value = fileStore.filters.onlyTagged;
  hideTagged.value = fileStore.filters.hideTagged;
  onlyLocated.value = fileStore.filters.onlyLocated;
  hideLocated.value = fileStore.filters.hideLocated;
  onlyError.value = fileStore.filters.onlyError;
  hideDuplicates.value = fileStore.filters.hideDuplicates;
  resizeGrid();
  window.addEventListener('resize', resizeGrid);
});

onUnmounted(() => {
  window.removeEventListener('resize', resizeGrid);
});
</script>

<template>
  <div class="controls">
    <v-btn icon @click="searchDialog = true" flat>
      <v-icon>mdi-filter</v-icon>
    </v-btn>
    <v-menu>
      <template v-slot:activator="{ props }">
        <v-btn v-bind="props" icon flat>
          <v-icon>mdi-sort</v-icon>
        </v-btn>
      </template>
      <v-list>
        <v-list-item
          @click="
            () => {
              sortBy = 0;
              sortDir = 1;
            }
          "
          >Sort by name (asc)</v-list-item
        >
        <v-list-item
          @click="
            () => {
              sortBy = 0;
              sortDir = -1;
            }
          "
          >Sort by name (desc)</v-list-item
        >
        <v-list-item
          @click="
            () => {
              sortBy = 1;
              sortDir = 1;
            }
          "
          >Sort by rating (asc)</v-list-item
        >
        <v-list-item
          @click="
            () => {
              sortBy = 1;
              sortDir = -1;
            }
          "
          >Sort by rating (desc)</v-list-item
        >
        <v-list-item
          @click="
            sortBy = 2;
            sortDir = 1;
          "
        >
          Sort by date
        </v-list-item>
      </v-list>
    </v-menu>
    <v-checkbox
      color="primary"
      class="collection-control"
      density="compact"
      v-model="selectMultiple"
      label="Select Multiple"
      @update:model-value="
        () => {
          if (!selectMultiple) {
            selected = [];
            $emit('select', selected);
          }
        }
      "
    ></v-checkbox>
    <v-btn
      icon
      flat
      @click="
        () => {
          itemsPerRow += 1;
          resizeGrid();
        }
      "
    >
      <v-icon>mdi-minus</v-icon>
    </v-btn>
    <v-btn
      icon
      flat
      @click="
        () => {
          if (itemsPerRow > 1) {
            itemsPerRow -= 1;
          }
          resizeGrid();
        }
      "
    >
      <v-icon>mdi-plus</v-icon>
    </v-btn>
    <v-menu v-if="selected.length > 1">
      <template v-slot:activator="{ props }">
        <v-btn v-bind="props" icon>
          <v-icon>mdi-dots-vertical</v-icon>
        </v-btn>
      </template>
      <v-list>
        <v-list-item
          @click="
            async () => {
              const groupName = selected[0].data.name;
              await addGroup(groupName);
              const target = [...selected];
              selected = [];
              target.forEach(async (photo) => {
                await setGroup(photo.data.name, groupName);
              });
            }
          "
          >Quick Group</v-list-item
        >
        <v-list-item
          @click="
            () => {
              targetTag = [];
              tagAddDialog = true;
            }
          "
          >Add Tags To Selected</v-list-item
        >
        <v-list-item
          @click="
            () => {
              targetTag = [];
              replacementTag = [];
              tagReplaceDialog = true;
            }
          "
          >Remove/Replace Tag From Selected</v-list-item
        >
      </v-list>
    </v-menu>
  </div>
  Showing {{ visiblePhotoCount }} / {{ photoCount }} photos
  <v-virtual-scroll
    :height="rows * size + 8"
    :item-height="size"
    :items="filteredPhotos"
    ref="gridCol"
  >
    <template v-slot:default="{ item }">
      <photo-icon
        v-for="(photo, i) in item"
        :key="i"
        :photo="photo"
        :size="size"
        :selected="selected.findIndex((p) => p.data.name === photo.data.name) >= 0"
        :invalid="!photo.valid"
        @select="selectPhoto(photo)"
      ></photo-icon>
    </template>
  </v-virtual-scroll>
  <v-dialog v-model="searchDialog">
    <v-card>
      <v-card-text>
        <tag-input
          label="Tags to include"
          :value="enabledTags"
          @update="(tags) => setFilter('enabledTags', tags)"
        ></tag-input>
        <v-select :items="['AND', 'OR']" label="Mode" v-model="filters.filterMode"></v-select>
        <tag-input
          label="Tags to exclude"
          :value="disabledTags"
          @update="(tags) => setFilter('disabledTags', tags)"
        ></tag-input>
        <v-checkbox
          v-model="onlyTagged"
          label="Only Show Tagged"
          @update:model-value="
            () => {
              if (onlyTagged) {
                hideTagged = false;
                setFilter('hideTagged', false);
              }
              setFilter('onlyTagged', onlyTagged);
            }
          "
        ></v-checkbox>
        <v-checkbox
          v-model="hideTagged"
          label="Hide Tagged"
          @update:model-value="
            () => {
              if (hideTagged) {
                onlyTagged = false;
                setFilter('onlyTagged', false);
              }
              setFilter('hideTagged', hideTagged);
            }
          "
        ></v-checkbox>
        <v-checkbox
          v-model="onlyLocated"
          label="Show Only Located"
          @update:model-value="
            () => {
              if (onlyLocated) {
                hideLocated = false;
                setFilter('hideLocated', false);
              }
              setFilter('onlyLocated', onlyLocated);
            }
          "
        ></v-checkbox>
        <v-checkbox
          v-model="hideLocated"
          label="Hide Located"
          @update:model-value="
            () => {
              if (hideLocated) {
                onlyLocated = false;
                setFilter('onlyLocated', false);
              }
              setFilter('hideLocated', hideLocated);
            }
          "
        ></v-checkbox>
        <v-checkbox
          v-model="onlyError"
          label="Show Only Photos With Errors"
          @update:model-value="() => setFilter('onlyError', onlyError)"
        ></v-checkbox>
        <v-checkbox
          v-model="hideDuplicates"
          label="Hide Duplicates"
          @update:model-value="() => setFilter('hideDuplicates', hideDuplicates)"
        ></v-checkbox>
      </v-card-text>
    </v-card>
  </v-dialog>
  <v-dialog v-model="tagAddDialog">
    <v-card>
      <v-card-title>Add Tags</v-card-title>
      <v-card-text>
        Add a tag to selected photos (<b>this action will effect {{ selected.length }} photos</b>!)
        <tag-input
          label="Tag to add"
          single
          :value="targetTag"
          @update="(tag) => (targetTag = tag)"
        ></tag-input>
        <v-card-actions>
          <v-btn
            color="primary"
            @click="
              async () => {
                loading = true;
                selected.forEach(async (photo) => {
                  await updateTagsForGroup(photo.data.name, photo.tags.concat(targetTag));
                });
                loading = false;
                tagAddDialog = false;
              }
            "
            :loading="loading"
            >Apply Changes</v-btn
          >
          <v-btn @click="tagAddDialog = false">Cancel</v-btn>
        </v-card-actions>
      </v-card-text>
    </v-card>
  </v-dialog>
  <v-dialog v-model="tagReplaceDialog">
    <v-card>
      <v-card-title>Remove and Replace Tags</v-card-title>
      <v-card-text>
        Search for a tag to remove (<b>this action will effect {{ selected.length }} photos</b>!)
        <tag-input
          label="Tag to find"
          :value="targetTag"
          @update="(tag) => (targetTag = tag)"
        ></tag-input>
        <v-radio-group v-model="tagAction">
          <v-radio label="Remove tag" value="remove"></v-radio>
          <v-radio label="Replace tag" value="replace"></v-radio>
        </v-radio-group>
        <div v-if="tagAction === 'replace'">
          Replace with:
          <tag-input
            label="Tag to replace with"
            :value="replacementTag"
            @update="(tag) => (replacementTag = tag)"
          ></tag-input>
          Replacing {{ targetTag[0] }} with {{ replacementTag[0] }}.
        </div>
      </v-card-text>
      <v-card-actions>
        <v-btn
          color="primary"
          @click="
            async () => {
              loading = true;
              selected.forEach(async (photo) => {
                if (tagAction === 'remove') {
                  const updatedTags = [...photo.tags];
                  updatedTags.splice(updatedTags.indexOf(targetTag[0]), 1);
                  await updateTagsForGroup(photo.data.name, updatedTags);
                } else {
                  const updatedTags = [...photo.tags];
                  updatedTags.splice(updatedTags.indexOf(targetTag[0]), 1);
                  updatedTags.push(replacementTag[0]);
                  await updateTagsForGroup(photo.data.name, updatedTags);
                }
              });
              loading = false;
              tagReplaceDialog = false;
            }
          "
          :loading="loading"
          >Apply Changes</v-btn
        >
        <v-btn @click="tagReplaceDialog = false">Cancel</v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<style scoped>
.controls {
  display: flex;
}

.collection-control {
  margin-top: 4px;
}
</style>
