<script setup lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onMounted, onUnmounted, ref } from 'vue';
  import { Photo, type PhotoData } from '../classes/Photo';
  import { fileStore } from '../stores/fileStore';

  type PhotoGridResponse = {
    photos: PhotoData[];
    total: number;
  };

  const props = defineProps<{
    photos: Photo[];
  }>();

  const emit = defineEmits<{
    (e: 'select', photos: Photo[]): void;
  }>();

  const { addGroup, photoCount, sort, setSortMode } = fileStore;

  const photoScroller = ref<HTMLDivElement | undefined>();
  const fakeScroller = ref<HTMLDivElement | undefined>();
  const spacerBottom = ref(0);
  const displayPhotos = ref<Photo[]>([]);
  const selectMultiple = ref(false);
  const selected = ref<Photo[]>([]);
  const itemsPerRow = ref(4);
  const size = ref(0);
  const sortBy = ref(0);
  const sortDir = ref(1);
  const currentRow = ref(0);

  /**
   * Resizes the grid items when the window size changes
   */
  function resizeGrid() {
    size.value = (photoScroller.value?.getBoundingClientRect()?.width ?? 0) / itemsPerRow.value - 8;
  }

  const rows = 8;

  /**
   * Handles selecting one or more photos.
   * @param photo - The photo being selected.
   */
  function selectPhoto(photo: Photo) {
    const s: Photo[] = [photo];
    if (photo.group) {
      // s = getByGroup(photo.group);
    }
    if (selectMultiple.value) {
      for (const x of s) {
        const idx = selected.value.findIndex(p => p.name === x.name);
        if (idx !== -1) {
          selected.value.splice(idx, 1);
        } else {
          selected.value.push(x);
        }
      }
    } else {
      selected.value = s;
    }
    emit('select', selected.value as Photo[]);
  }

  // Tag add/replace dialogs
  const tagReplaceDialog = ref(false);
  const tagAddDialog = ref(false);
  const targetTags = ref<string[]>([]);
  const tagAction = ref<'remove' | 'replace'>('remove');
  const replacementTag = ref<string[]>([]);
  const loading = ref(false);

  onMounted(async () => {
    sortBy.value = sort[0] ?? 0;
    sortDir.value = sort[1] ?? 0;
    resizeGrid();
    window.addEventListener('resize', resizeGrid);
    if (fakeScroller.value !== undefined) {
      const { photos, total } = await invoke<PhotoGridResponse>('photo_grid', {
        start: 0,
        count: itemsPerRow.value * rows,
      });
      spacerBottom.value = ((total - itemsPerRow.value * rows) / itemsPerRow.value) * size.value;
      displayPhotos.value = Photo.createPhotos(photos);
      fakeScroller.value.addEventListener('scroll', async ev => {
        const scroll = (ev.target as HTMLDivElement).scrollTop;
        const scrolledToRow = Math.floor(scroll / size.value);
        if (scrolledToRow < currentRow.value) {
          currentRow.value = scrolledToRow;
          const { photos } = await invoke<PhotoGridResponse>('photo_grid', {
            start: scrolledToRow * itemsPerRow.value,
            count: itemsPerRow.value,
          });
          displayPhotos.value.unshift(...Photo.createPhotos(photos));
          displayPhotos.value.splice(displayPhotos.value.length - itemsPerRow.value);
        } else if (scrolledToRow > currentRow.value) {
          currentRow.value = scrolledToRow;
          const { photos } = await invoke<PhotoGridResponse>('photo_grid', {
            start: (scrolledToRow + rows) * itemsPerRow.value,
            count: itemsPerRow.value,
          });
          displayPhotos.value.splice(0, itemsPerRow.value);
          displayPhotos.value.push(...Photo.createPhotos(photos));
        }
      });
    }
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
            // $emit('select', selected);
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
              const groupName = selected[0]?.name;
              if (groupName) {
                await addGroup(groupName);
                const target = [...selected];
                selected = [];
                target.forEach(async photo => {
                  // await setGroup(photo.name, groupName);
                });
              }
            }
          "
          >Quick Group</v-list-item
        >
        <v-list-item
          @click="
            () => {
              targetTags = [];
              tagAddDialog = true;
            }
          "
          >Add Tags To Selected</v-list-item
        >
        <v-list-item
          @click="
            () => {
              targetTags = [];
              replacementTag = [];
              tagReplaceDialog = true;
            }
          "
          >Remove/Replace Tag From Selected</v-list-item
        >
      </v-list>
    </v-menu>
  </div>
  Showing {{ displayPhotos.length }} / {{ photoCount }} photos
  <div class="scroller-container">
    <div class="photo-scroller" ref="photoScroller" :style="{ height: `${rows * size}px` }">
      <photo-icon
        v-for="(photo, i) in displayPhotos"
        :key="i"
        :photo="photo as Photo"
        :size="size"
        :selected="selected.findIndex(p => p.name === photo.name) >= 0"
        :invalid="!photo.valid"
        @select="selectPhoto(photo as Photo)"
      />
    </div>
    <div
      class="fake-scroll"
      :style="{
        height: `${rows * size}px`,
        top: `-${rows * size}px`,
        width: `${12 + itemsPerRow * size}px`,
      }"
      ref="fakeScroller"
      @click="event => {
        selectPhoto(displayPhotos[(itemsPerRow * Math.floor(event.offsetY / size)) + Math.floor(event.offsetX / size) - (itemsPerRow * currentRow)] as Photo);
      }"
    >
      <div class="spacer" :style="{ height: `${spacerBottom}px` }" />
    </div>
  </div>
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
          $emit('select', selected as Photo[]);
        }
      }
    "
  />
  <v-dialog v-model="tagAddDialog">
    <v-card>
      <v-card-title>Add Tags</v-card-title>
      <v-card-text>
        Add a tag to selected photos (<b>this action will effect {{ selected.length }} photos</b>!)
        <tag-input
          label="Tag to add"
          :value="targetTags"
          @update="tags => (targetTags = tags)"
        ></tag-input>
        <v-card-actions>
          <v-btn
            color="primary"
            @click="
              async () => {
                loading = true;
                for (const photo of selected) {
                  const combinedTags = new Set(photo.tags);
                  for (const tag of targetTags) {
                    combinedTags.add(tag);
                  }
                  await photo.setTags(combinedTags.values().toArray());
                }
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
          :value="targetTags"
          @update="tag => (targetTags = tag)"
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
            @update="tag => (replacementTag = tag)"
          ></tag-input>
          Replacing {{ targetTags[0] }} with {{ replacementTag[0] }}.
        </div>
      </v-card-text>
      <v-card-actions>
        <v-btn
          color="primary"
          @click="
            async () => {
              loading = true;
              const tag = targetTags[0];
              if (tag) {
                for (const photo of selected) {
                  if (tagAction === 'remove') {
                    const updatedTags = [...photo.tags];
                    updatedTags.splice(updatedTags.indexOf(tag), 1);
                    await photo.setTags(updatedTags);
                  } else {
                    const updatedTags = [...photo.tags];
                    updatedTags.splice(updatedTags.indexOf(tag), 1);
                    if (replacementTag[0]) {
                      updatedTags.push(replacementTag[0]);
                    }
                    await photo.setTags(updatedTags);
                  }
                }
              }
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

  .fake-scroll {
    overflow-y: scroll;
    position: relative;
  }

  .photo-scroller {
    display: flex;
    flex-wrap: wrap;
    overflow: hidden;
  }
</style>
