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

  const { addGroup, photoCount } = fileStore;

  const photoScroller = ref<HTMLDivElement | undefined>();
  const fakeScroller = ref<HTMLDivElement | undefined>();
  const spacer = ref(0);
  const displayPhotos = ref<Photo[]>([]);
  const selectMultiple = ref(false);
  const selected = ref<Photo[]>([]);
  const itemsPerRow = ref(4);
  const size = ref(0);
  const localQuery = ref<string[]>([]);
  const sortBy = ref('name'); // This variable will be used when searching is fixed
  const currentRow = ref(0);

  async function search() {
    await invoke('search_photos', {
      query: localQuery.value,
      sort: sortBy.value,
    });
    const { photos, total } = await invoke<PhotoGridResponse>('photo_grid', {
      start: 0,
      count: itemsPerRow.value * rows,
    });
    displayPhotos.value = Photo.createPhotos(photos);
    spacer.value = ((total - itemsPerRow.value * rows) / itemsPerRow.value) * size.value;
  }

  defineExpose({
    async search(query: string[]) {
      localQuery.value = query;
      await search();
    },
    async updateRating() {
      if (sortBy.value === 'rating' || sortBy.value === 'rating_desc') {
        await search();
      }
    }
  });

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
    let s: Photo[] = [photo];
    if (photo.group) {
      s = getByGroup(photo.group);
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

  async function setSortMode(sort: string) {
    sortBy.value = sort;
    await search();
  }

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

  onMounted(async () => {
    resizeGrid();
    window.addEventListener('resize', resizeGrid);
    if (fakeScroller.value !== undefined) {
      const { photos, total } = await invoke<PhotoGridResponse>('photo_grid', {
        start: 0,
        count: itemsPerRow.value * rows,
      });
      spacer.value = ((total - itemsPerRow.value * rows) / itemsPerRow.value) * size.value;
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
        <v-list-item @click="setSortMode('name')">Sort by name</v-list-item>
        <v-list-item @click="setSortMode('name_desc')">Sort by name (desc)</v-list-item>
        <v-list-item @click="setSortMode('rating')">Sort by rating</v-list-item>
        <v-list-item @click="setSortMode('rating_desc')"> Sort by rating (desc) </v-list-item>
        <v-list-item @click="setSortMode('date')">Sort by date</v-list-item>
        <v-list-item @click="setSortMode('date_desc')">Sort by date (desc)</v-list-item>
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
    />
    <v-btn @click="selected = [...props.photos]"> Select All </v-btn>
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
                  await setGroup(photo.name, groupName);
                });
              }
            }
          "
        >
          Quick Group
        </v-list-item>
        <v-list-item
          @click="
            () => {
              targetTags = [];
              tagAddDialog = true;
            }
          "
        >
          Add Tags To Selected
        </v-list-item>
        <v-list-item
          @click="
            () => {
              targetTags = [];
              replacementTag = [];
              tagReplaceDialog = true;
            }
          "
        >
          Remove/Replace Tag From Selected
        </v-list-item>
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
      @click="event => selectPhoto(displayPhotos[(itemsPerRow * Math.floor(event.offsetY / size)) + Math.floor(event.offsetX / size) - (itemsPerRow * currentRow)] as Photo)"
    >
      <div class="spacer" :style="{ height: `${spacer}px` }" />
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
        <tag-input label="Tag to add" :value="targetTags" @update="tags => (targetTags = tags)" />
        <v-card-actions>
          <v-btn color="primary" @click="addTags()" :loading="loading"> Apply Changes </v-btn>
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
        <tag-input label="Tag to find" :value="targetTags" @update="tag => (targetTags = tag)" />
        <v-radio-group v-model="tagAction">
          <v-radio label="Remove tag" value="remove" />
          <v-radio label="Replace tag" value="replace" />
        </v-radio-group>
        <div v-if="tagAction === 'replace'">
          Replace with:
          <tag-input
            label="Tag to replace with"
            :value="replacementTag"
            @update="tag => (replacementTag = tag)"
          />
          Replacing {{ targetTags[0] }} with {{ replacementTag[0] }}.
        </div>
      </v-card-text>
      <v-card-actions>
        <v-btn color="primary" @click="replaceTags()" :loading="loading"> Apply Changes </v-btn>
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
