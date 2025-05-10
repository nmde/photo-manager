<script setup lang="ts">
import { computed, ref, onMounted, onUnmounted } from 'vue';
import { Photo } from '../classes/Photo';
import { fileStore } from '../stores/fileStore';
import PhotoIcon from './PhotoIcon.vue';
import TagInput from './TagInput.vue';

const props = defineProps<{
  photos: Photo[];
}>();

const emit = defineEmits<{
  (e: 'select', photos: Photo[]): void;
}>();

const { getByGroup, addGroup, setGroup, photoCount, updateTagsForGroup, sort, setSortMode } =
  fileStore;

const selectMultiple = ref(false);
const selected = ref<Photo[]>([]);
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
  return 8;
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
const targetTags = ref<string[]>([]);
const tagAction = ref<'remove' | 'replace'>('remove');
const replacementTag = ref<string[]>([]);
const loading = ref(false);

onMounted(() => {
  sortBy.value = sort[0];
  sortDir.value = sort[1];
  resizeGrid();
  window.addEventListener('resize', resizeGrid);
});

onUnmounted(() => {
  window.removeEventListener('resize', resizeGrid);
});
</script>

<template>
  <div class="controls">
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
              setSortMode(0, 1);
            }
          "
          >Sort by name (asc)</v-list-item
        >
        <v-list-item
          @click="
            () => {
              sortBy = 0;
              sortDir = -1;
              setSortMode(0, -1);
            }
          "
          >Sort by name (desc)</v-list-item
        >
        <v-list-item
          @click="
            () => {
              sortBy = 1;
              sortDir = 1;
              setSortMode(1, 1);
            }
          "
          >Sort by rating (asc)</v-list-item
        >
        <v-list-item
          @click="
            () => {
              sortBy = 1;
              sortDir = -1;
              setSortMode(1, -1);
            }
          "
          >Sort by rating (desc)</v-list-item
        >
        <v-list-item
          @click="
            sortBy = 2;
            sortDir = 1;
            setSortMode(2, 1);
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
      @click="
        () => {
          selected = [...props.photos];
        }
      "
      >Select All</v-btn
    >
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
  <v-dialog v-model="tagAddDialog">
    <v-card>
      <v-card-title>Add Tags</v-card-title>
      <v-card-text>
        Add a tag to selected photos (<b>this action will effect {{ selected.length }} photos</b>!)
        <tag-input
          label="Tag to add"
          :value="targetTag"
          @update="(tags) => (targetTags = tags)"
        ></tag-input>
        <v-card-actions>
          <v-btn
            color="primary"
            @click="
              async () => {
                loading = true;
                selected.forEach(async (photo) => {
                  await updateTagsForGroup(photo.data.name, photo.tags.concat(...targetTags));
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
