<script setup lang="ts">
import { storeToRefs } from 'pinia';
import { computed, ref } from 'vue';
import { Photo } from '../classes/Photo';
import { useFileStore } from '../stores/fileStore';

const props = defineProps<{
  photos: Photo[];
  itemsPerRow: number;
  size: number;
}>();

const emit = defineEmits<{
  (e: 'select', photos: Photo[]): void;
}>();

const store = useFileStore();
const { getByGroup, validateTags, updateTags } = store;
const { photoCount, filters } = storeToRefs(store);

const selectMultiple = ref(false);
const selected = ref<Photo[]>([]);
const searchDialog = ref(false);
const rowCount = ref(0);

/**
 * Adjusts the row count to fill the window height
 */
function adjustRows() {
  rowCount.value = Math.ceil(document.documentElement.scrollHeight / props.size);
}

type GridRow = Photo[];

// Filters the photos based on the options
const filteredPhotos = computed(() => {
  const rows: GridRow[] = [];
  let row: GridRow = [];
  const tempphotos = [...props.photos];
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
      if (row.length === props.itemsPerRow) {
        rows.push(row);
        row = [];
      }
    }
    tempphotos.shift();
  }
  rows.push(row);
  return rows;
});

// The number of visible photos after filter rules are applied
const visiblePhotoCount = computed(() => {
  return (
    (filteredPhotos.value.length - 1) * props.itemsPerRow +
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

onMounted(() => {
  adjustRows();
  window.addEventListener('resize', adjustRows);
});

onUnmounted(() => {
  window.removeEventListener('resize', adjustRows);
});

// Tag replace dialog
const tagReplaceDialog = ref(false);
const targetTag = ref<string[]>([]);
const tagAction = ref<'remove' | 'replace'>('remove');
const replacementTag = ref<string[]>([]);
const loading = ref(false);
</script>

<template>
  <div class="controls">
    <v-btn icon @click="searchDialog = true" flat>
      <v-icon>mdi-filter</v-icon>
    </v-btn>
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
    <v-menu v-if="selected.length > 1">
      <template v-slot:activator="{ props }">
        <v-btn v-bind="props" icon>
          <v-icon>mdi-dots-vertical</v-icon>
        </v-btn>
      </template>
      <v-list>
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
    :height="rowCount * props.size"
    :item-height="props.size"
    :items="filteredPhotos"
  >
    <template v-slot:default="{ item }">
      <photo-icon
        v-for="(photo, i) in item"
        :key="i"
        :photo="photo"
        :size="props.size"
        :selected="selected.findIndex((p) => p.data.name === photo.data.name) >= 0"
        :invalid="validateTags(photo.data.name) !== null"
        @select="selectPhoto(photo)"
      ></photo-icon>
    </template>
  </v-virtual-scroll>
  <v-dialog v-model="searchDialog">
    <v-card>
      <v-card-text>
        <tag-input
          label="Tags to include"
          :value="filters.enabledTags"
          @update="(tags) => (filters.enabledTags = tags)"
        ></tag-input>
        <v-select :items="['AND', 'OR']" label="Mode" v-model="filters.filterMode"></v-select>
        <tag-input
          label="Tags to exclude"
          :value="filters.disabledTags"
          @update="(tags) => (filters.disabledTags = tags)"
        ></tag-input>
        <v-checkbox
          v-model="filters.onlyTagged"
          label="Only Show Tagged"
          @update:model-value="
            () => {
              if (filters.onlyTagged) {
                filters.hideTagged = false;
              }
            }
          "
        ></v-checkbox>
        <v-checkbox
          v-model="filters.hideTagged"
          label="Hide Tagged"
          @update:model-value="
            () => {
              if (filters.hideTagged) {
                filters.onlyTagged = false;
              }
            }
          "
        ></v-checkbox>
        <v-checkbox
          v-model="filters.onlyLocated"
          label="Show Only Located"
          @update:model-value="
            () => {
              if (filters.onlyLocated) {
                filters.hideLocated = false;
              }
            }
          "
        ></v-checkbox>
        <v-checkbox
          v-model="filters.hideLocated"
          label="Hide Located"
          @update:model-value="
            () => {
              if (filters.hideLocated) {
                filters.onlyLocated = false;
              }
            }
          "
        ></v-checkbox>
        <v-checkbox v-model="filters.onlyError" label="Show Only Photos With Errors"></v-checkbox>
        <v-checkbox v-model="filters.hideDuplicates" label="Hide Duplicates"></v-checkbox>
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
          single
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
            single
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
                  await updateTags(photo.data.name, updatedTags);
                } else {
                  const updatedTags = [...photo.tags];
                  updatedTags.splice(updatedTags.indexOf(targetTag[0]), 1);
                  updatedTags.push(replacementTag[0]);
                  await updateTags(photo.data.name, updatedTags);
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
