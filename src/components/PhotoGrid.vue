<script setup lang="ts">
import { storeToRefs } from 'pinia';
import { computed, ref } from 'vue';
import { Photo } from '../classes/Photo';
import { useFileStore } from '../stores/fileStore';

const props = defineProps<{
  photos: Photo[];
  itemsPerRow: number;
  size: number;
  rows: number;
}>();

const emit = defineEmits<{
  (e: 'select', photos: Photo[]): void;
}>();

const { photoCount } = storeToRefs(useFileStore());

const hideTagged = ref(false);
const hideLocated = ref(false);
const hideDuplicate = ref(false);
const selectMultiple = ref(false);
const selected = ref<Photo[]>([]);

type GridRow = Photo[];

// Filters the photos based on the options
const filteredPhotos = computed(() => {
  const rows: GridRow[] = [];
  let row: GridRow = [];
  const tempphotos = [...props.photos];
  const groups: string[] = [];
  while (tempphotos.length > 0) {
    const file = tempphotos[0];
    let visible = true;
    if (hideTagged.value === true && file.tags.length > 0) {
      visible = false;
    }
    if (hideLocated.value === true && file.location !== undefined) {
      visible = false;
    }
    if (hideDuplicate.value === true && file.data.isDuplicate) {
      visible = false;
    }
    if (visible) {
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
  if (selectMultiple.value) {
    const idx = selected.value.findIndex((p) => p.data.name === photo.data.name);
    if (idx >= 0) {
      selected.value.splice(idx, 1);
    } else {
      selected.value.push(photo);
    }
  } else {
    selected.value = [photo];
  }
  emit('select', selected.value);
}
</script>

<template>
  <div class="controls">
    <v-menu :close-on-content-click="false">
      <template v-slot:activator="{ props }">
        <v-btn icon v-bind="props" flat>
          <v-icon>mdi-dots-vertical</v-icon>
        </v-btn>
      </template>
      <v-list>
        <v-list-item>
          <v-checkbox
            density="compact"
            v-model="hideTagged"
            label="Hide tagged"
          ></v-checkbox>
        </v-list-item>
        <v-list-item>
          <v-checkbox
            density="compact"
            v-model="hideLocated"
            label="Hide located"
          ></v-checkbox>
        </v-list-item>
        <v-list-item>
          <v-checkbox
            density="compact"
            v-model="hideDuplicate"
            label="Hide duplicates"
          ></v-checkbox>
        </v-list-item>
      </v-list>
    </v-menu>
    <v-checkbox
      color="primary"
      class="collection-control"
      density="compact"
      v-model="selectMultiple"
      label="Select Multiple"
      @update:model-value="() => {
        if (!selectMultiple) {
          selected = [];
          $emit('select', selected);
        }
      }"
    ></v-checkbox>
  </div>
  Showing {{ visiblePhotoCount }} / {{ photoCount }} photos
  <v-virtual-scroll
    :height="props.rows * props.size"
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
        @select="selectPhoto(photo)"
      ></photo-icon>
    </template>
  </v-virtual-scroll>
</template>

<style scoped>
.controls {
  display: flex;
}

.collection-control {
  margin-top: 4px;
}
</style>
