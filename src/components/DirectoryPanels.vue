<script setup lang="ts">
import type { Photo } from '../classes/Photo';
import { fileStore } from '../stores/fileStore';
import PhotoGrid from './PhotoGrid.vue';

const { files } = fileStore;

type Folder = {
  files: string[];
  children: Record<string, Folder>;
};

defineProps<{
  folderStructure: Folder;
}>();

const emit = defineEmits<{
  (e: 'select', s: Photo[]): void;
}>();

// TODO: photos in the root level directory are not displayed
</script>

<template>
  <v-expansion-panels>
    <v-expansion-panel v-for="dir in Object.keys(folderStructure.children)" :key="dir" :title="dir">
      <v-expansion-panel-text>
        <div v-if="Object.keys(folderStructure.children[dir]).length > 0">
          <directory-panels
            :folder-structure="folderStructure.children[dir] ?? { files: [], children: {} }"
            @select="s => emit('select', s)"
          />
        </div>
        <photo-grid
          :photos="
            folderStructure.children[dir]?.files.map(s => files[s]).filter(s => s !== undefined) ??
              []
          "
          @select="s => emit('select', s)"
        />
      </v-expansion-panel-text>
    </v-expansion-panel>
  </v-expansion-panels>
</template>
