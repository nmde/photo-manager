<script setup lang="ts">
import { storeToRefs } from 'pinia';
import { computed, ref } from 'vue';
import { Photo } from '../../classes/Photo';
import { useFileStore } from '../../stores/fileStore';

const fileStore = useFileStore();
const { validateTags } = fileStore;
const { files } = storeToRefs(fileStore);

const selected = ref<Photo[]>([]);
const enabledTags = ref<string[]>([]);
const disabledTags = ref<string[]>([]);
const searchDialog = ref(false);
const includeMode = ref('AND');
const hideTagged = ref(false);
const onlyTagged = ref(false);
const onlyError = ref(false);
const hideDuplicates = ref(true);
const onlyLocated = ref(false);
const hideLocated = ref(false);

const photos = computed(() => {
  const filtered: Photo[] = [];
  Object.values(files.value).forEach((file) => {
    let satisfiesTags = includeMode.value === 'AND' || enabledTags.value.length === 0;
    enabledTags.value.forEach((tag) => {
      if (includeMode.value === 'OR' && file.tags.indexOf(tag) >= 0) {
        satisfiesTags = true;
      } else if (includeMode.value === 'AND' && file.tags.indexOf(tag) < 0) {
        satisfiesTags = false;
      }
    });
    disabledTags.value.forEach((tag) => {
      if (file.tags.indexOf(tag) >= 0) {
        satisfiesTags = false;
      }
    });
    if (hideTagged.value === true && file.tags.length > 0) {
      satisfiesTags = false;
    }
    if (onlyTagged.value === true && file.tags.length === 0) {
      satisfiesTags = false;
    }
    if (hideLocated.value === true && file.location !== undefined) {
      satisfiesTags = false;
    }
    if (onlyLocated.value === true && file.location === undefined) {
      satisfiesTags = false;
    }
    if (onlyError.value === true && validateTags(file.data.name) === null) {
      satisfiesTags = false;
    }
    if (hideDuplicates.value === true && file.data.isDuplicate) {
      satisfiesTags = false;
    }
    if (satisfiesTags) {
      filtered.push(file);
    }
  });
  return filtered;
});
</script>

<template>
  <v-main class="main">
    <v-container fluid>
      <v-row>
        <v-col cols="6">
          <div class="flex">
            <tag-input
              label="Tags to include"
              :value="enabledTags"
              @update="(tags) => (enabledTags = tags)"
            ></tag-input>
            <v-btn @click="searchDialog = true">Advanced</v-btn>
          </div>
          <photo-grid
            :photos="photos"
            :items-per-row="4"
            @select="(s) => (selected = s)"
            :size="170"
            :rows="4"
          ></photo-grid>
        </v-col>
        <v-col cols="6">
          <photo-group v-if="selected.length > 0" :photos="selected"></photo-group>
        </v-col>
      </v-row>
    </v-container>
    <v-dialog v-model="searchDialog">
      <v-card>
        <v-card-text>
          <tag-input
            label="Tags to include"
            :value="enabledTags"
            @update="(tags) => (enabledTags = tags)"
          ></tag-input>
          <v-select :items="['AND', 'OR']" label="Mode" v-model="includeMode"></v-select>
          <tag-input
            label="Tags to exclude"
            :value="disabledTags"
            @update="(tags) => (disabledTags = tags)"
          ></tag-input>
          <v-checkbox
            v-model="onlyTagged"
            label="Only Show Tagged"
            @update:model-value="
              () => {
                if (onlyTagged) {
                  hideTagged = false;
                }
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
                }
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
                }
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
                }
              }
            "
          ></v-checkbox>
          <v-checkbox v-model="onlyError" label="Show Only Photos With Errors"></v-checkbox>
          <v-checkbox v-model="hideDuplicates" label="Hide Duplicates"></v-checkbox>
        </v-card-text>
      </v-card>
    </v-dialog>
  </v-main>
</template>

<style scoped></style>
