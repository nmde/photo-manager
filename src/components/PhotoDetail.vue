<script setup lang="ts">
import { VideoPlayer } from '@videojs-player/vue';
import { storeToRefs } from 'pinia';
import 'video.js/dist/video-js.css';
import { computed, ref } from 'vue';
import { Photo } from '../classes/Photo';
import { useFileStore } from '../stores/fileStore';

const fileStore = useFileStore();
const { addGroup, removeGroup } = fileStore;
const { groupNames, tags } = storeToRefs(fileStore);

const emit = defineEmits<{
  (e: 'update:title', title: string): void;
  (e: 'update:description', description: string): void;
  (e: 'update:tags', tags: string[]): void;
  (e: 'update:rating', rating: number): void;
  (e: 'update:isDuplicate', isDuplicate: boolean): void;
  (e: 'update:group', group?: string): void;
  (e: 'update:date', date: string): void;
}>();

const props = defineProps<{
  photo: Photo;
}>();

const photoPath = computed(() => {
  if (props.photo.data.thumbnail.length > 0) {
    return props.photo.data.thumbnail;
  }
  return props.photo.data.path;
});

const showAddGroup = ref(false);
const newGroupName = ref('');
const rating = ref(0);
const isDuplicate = ref(false);
const group = ref<string | undefined>();
const photoTags = ref<string[]>([]);
const title = ref('');
const description = ref('');
const newGroupError = ref(false);
const date = ref('');
const closeUp = ref(false);

function initialize() {
  rating.value = props.photo.data.rating;
  isDuplicate.value = props.photo.data.isDuplicate;
  group.value = props.photo.group;
  photoTags.value = props.photo.tags;
  title.value = props.photo.data.title;
  description.value = props.photo.data.description;
  date.value = props.photo.data.date;
}

watch(() => props.photo, initialize);

onMounted(initialize);
</script>

<template>
  <video-player
    v-if="photo.data.video"
    :src="photo.data.path"
    :poster="photo.data.thumbnail"
    controls
    :width="700"
    :height="400"
  ></video-player>
  <v-img v-if="!photo.data.video" max-height="600" :src="photoPath" @click="closeUp = true"></v-img>
  <br />
  <tag-input
    advanced
    :label="`Photo Tags (${photoTags.length})`"
    :value="photoTags"
    filtered
    :validate="photo.data.name"
    @change="
      (tags) => {
        photoTags = tags;
      }
    "
    @update="
      (tags) => {
        emit('update:tags', photoTags);
      }
    "
    :target="photo.data.name"
  ></tag-input>
  <!--
  <v-btn
    @click="
      () => {
        photoTags = tags;
        emit('update:tags', tags);
      }
    "
    >Set All Tags</v-btn
  >
  <v-btn
    @click="
      () => {
        photoTags = [];
        emit('update:tags', []);
      }
    "
    >Clear Tags</v-btn
  >
  -->
  <v-select
    label="Group"
    :items="groupNames"
    v-model="group"
    @update:model-value="emit('update:group', group)"
  ></v-select>
  <v-text-field
    label="Title"
    v-model="title"
    @update:model-value="emit('update:title', title)"
  ></v-text-field>
  <v-textarea
    label="Description"
    v-model="description"
    @update:model-value="emit('update:description', description)"
  ></v-textarea>
  <v-text-field
    label="Date"
    v-model="date"
    @update:model-value="emit('update:date', date)"
  ></v-text-field>
  <v-rating v-model="rating" @update:model-value="emit('update:rating', rating)"></v-rating>
  <v-checkbox
    label="Mark as duplicate"
    v-model="isDuplicate"
    @update:model-value="emit('update:isDuplicate', isDuplicate)"
  ></v-checkbox>
  <v-btn icon @click="showAddGroup = !showAddGroup">
    <v-icon>mdi-plus</v-icon>
  </v-btn>
  <v-btn icon @click="removeGroup(photo.data.name)">
    <v-icon>mdi-trash-can</v-icon>
  </v-btn>
  <div v-if="showAddGroup">
    <v-text-field label="New Group Name" v-model="newGroupName"></v-text-field>
    <v-btn
      color="primary"
      :error="newGroupError"
      @click="
        () => {
          if (groupNames.indexOf(newGroupName) >= 0) {
            newGroupError = true;
          } else if (newGroupName.length > 0) {
            addGroup(newGroupName);
            newGroupName = '';
            newGroupError = false;
          }
        }
      "
      >Create Group</v-btn
    >
  </div>
  <v-dialog v-model="closeUp">
    <v-card>
      <v-card-title>{{ photo.data.title }}</v-card-title>
      <v-card-text>
        <video-player
          v-if="photo.data.video"
          :src="photo.data.path"
          :poster="photo.data.thumbnail"
          controls
          :width="700"
          :height="400"
        ></video-player>
        <v-img
          v-if="!photo.data.video"
          max-height="600"
          :src="photoPath"
        ></v-img>
      </v-card-text>
    </v-card>
  </v-dialog>
</template>
