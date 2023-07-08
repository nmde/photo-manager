<script setup lang="ts">
import { storeToRefs } from 'pinia';
import { ref } from 'vue';
import PhotoIcon from '../components/PhotoIcon.vue';
import { useFileStore } from '../stores/fileStore';

const { files, workingDir } = storeToRefs(useFileStore());

const showOnlyUntagged = ref(true);
const selected = ref('');

/**
 * Select a photo to edit.
 * @param index - The name of the photo.
 */
function selectPhoto(name: string) {
    selected.value = name;
}
</script>

<template>
    <v-main>
        <v-container fluid>
            <v-row>
                <v-col cols=6>
                    <h1>{{ workingDir }}</h1>
                    <v-checkbox v-model="showOnlyUntagged" label="Show only untagged"></v-checkbox>
                </v-col>
                <v-col cols="6">
                    <h1>{{ selected }}</h1>
                </v-col>
            </v-row>
            <v-row>
                <v-col cols="6">
                    <photo-icon v-for="(photo, i) in files" :key="i" :photo="photo" :size="150" @select="selectPhoto(i)"></photo-icon>
                </v-col>
                <v-col cols="6"></v-col>
            </v-row>
        </v-container>
    </v-main>
</template>
