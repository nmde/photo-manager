<script setup lang="ts">
import { storeToRefs } from 'pinia';
import { computed, ref, onMounted } from 'vue';
import { Photo } from '../classes/Photo';
import { useFileStore } from '../stores/fileStore';

const fileStore = useFileStore();
const { addGroup, setRating, setDuplicate, setGroup, removeGroup, updateTags } = fileStore;
const { groupNames, tags } = storeToRefs(fileStore);

const props = defineProps<{
  photo: Photo;
}>();

const photoPath = computed(() => {
  if (props.photo.data.thumbnail) {
    return props.photo.data.thumbnail;
  }
  return props.photo.data.path;
});

const showAddGroup = ref(false);
const newGroupName = ref('');

function createGroup() {
  if (newGroupName.value.length > 0) {
    addGroup(newGroupName.value, []);
    newGroupName.value = '';
  }
}

const rating = ref(0);
const isDuplicate = ref(false);
const group = ref('');
const photoTags = ref<string[]>([]);

onMounted(() => {
  if (props.photo.data.rating) {
    rating.value = props.photo.data.rating;
  }
  isDuplicate.value = props.photo.data.isDuplicate;
  if (props.photo.group) {
    group.value = props.photo.group;
  }
  photoTags.value = props.photo.tags;
});
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
  <v-img v-if="!photo.data.video" max-height="600" :src="photoPath"></v-img>
  <br />
  Title: {{ photo.data.title }} <br />
  Description: {{ photo.data.description }} <br />
  <v-combobox
    label="Photo Tags"
    :items="tags"
    multiple
    chips
    v-model="photoTags"
    @update:model-value="updateTags(photo.data.name, photoTags)"
  ></v-combobox>
  <v-rating v-model="rating" @update:model-value="setRating(photo.data.name, rating)"></v-rating>
  <v-checkbox
    label="Mark as duplicate"
    v-model="isDuplicate"
    @update:model-value="setDuplicate(photo.data.name, isDuplicate)"
  ></v-checkbox>
  <v-select
    label="Group"
    :items="groupNames"
    v-model="group"
    @update:model-value="setGroup(photo.data.name, group)"
  ></v-select>
  <v-btn icon @click="showAddGroup = !showAddGroup">
    <v-icon>mdi-plus</v-icon>
  </v-btn>
  <v-btn icon @click="removeGroup(photo.data.name)">
    <v-icon>mdi-trash-can</v-icon>
  </v-btn>
  <div v-if="showAddGroup">
    <v-text-field label="New Group Name" v-model="newGroupName"></v-text-field>
    <v-btn color="primary" @click="createGroup">Create Group</v-btn>
  </div>
</template>
