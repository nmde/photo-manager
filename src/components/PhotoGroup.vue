<script setup lang="ts">
import { storeToRefs } from 'pinia';
import { ref, computed } from 'vue';
import { useFileStore } from '../stores/fileStore';
import PhotoDetail from './PhotoDetail.vue';

const { groups, files } = storeToRefs(useFileStore());

const props = defineProps<{
    group: string;
}>();

const current = ref(0);

const groupItems = computed(() => {
    return groups.value[props.group];
});

function prev() {
    if (current.value > 0) {
        current.value -= 1;
    }
}

function next() {
    if (current.value < groupItems.value.length) {
        current.value += 1;
    }
}

</script>

<template>
  <v-btn icon @click="prev">
    <v-icon>mdi-arrow-left</v-icon>
  </v-btn>
  <v-btn icon @click="next">
    <v-icon>mdi-arrow-right</v-icon>
  </v-btn>
  <photo-detail :photo="files[groupItems[current]]"></photo-detail>
</template>
