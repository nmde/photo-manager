<script setup lang="ts">
import { VideoPlayer } from '@videojs-player/vue';
import 'video.js/dist/video-js.css';
import { computed, ref, watch, onMounted } from 'vue';
import { Photo } from '../classes/Photo';
import { fileStore } from '../stores/fileStore';
import AutosaveText from './AutosaveText.vue';
import PeopleInput from './PeopleInput.vue';
import TagInput from './TagInput.vue';

const { groupNames, addGroup, removeGroup, places, layers, setPersonPhoto, cameras } = fileStore;

const emit = defineEmits<{
  (e: 'update:title', title: string): void;
  (e: 'update:description', description: string): void;
  (e: 'update:tags', tags: string[]): void;
  (e: 'update:rating', rating: number): void;
  (e: 'update:isDuplicate', isDuplicate: boolean): void;
  (e: 'update:group', group?: string): void;
  (e: 'update:date', date: string): void;
  (e: 'update:location', location: string): void;
  (e: 'update:people', people: string[]): void;
  (e: 'update:photographer', photographer: string): void;
  (e: 'update:hideThumbnail', value: boolean): void;
  (e: 'update:camera', value: string): void;
}>();

const props = defineProps<{
  photo: Photo;
  prevDate: Date;
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
const date = ref<Date>(new Date());
const closeUp = ref(false);
const location = ref('');
const showRaw = ref(false);
const photoPeople = ref<string[]>([]);
const photographer = ref<string[]>([]);
const hideThumbnail = ref(false);
const focusDate = ref<Date>(new Date());
const camera = ref<string>('');

const setPhotoDialog = ref(false);
const setPhotoTarget = ref<string[]>([]);
const viewConfirmation = ref(false);

const placeList = computed(() => {
  return Object.values(places)
    .sort((a, b) => {
      if (a.isNewestPlace) {
        return -1;
      }
      if (b.isNewestPlace) {
        return 1;
      }
      return b.count - a.count;
    })
    .map((p) => ({
      color: layers[p.data.layer]?.data.color,
      title: `${p.data.name} (${p.count})`,
      value: p.Id,
    }));
});

const cameraList = computed(() => {
  return Object.values(cameras)
    .sort((a, b) => b.count - a.count)
    .map((x) => ({
      title: `${x.data.name} (${x.count})`,
      value: x.Id,
    }));
});

function initialize() {
  rating.value = props.photo.data.rating;
  isDuplicate.value = props.photo.data.isDuplicate;
  group.value = props.photo.group;
  photoTags.value = props.photo.tags;
  title.value = props.photo.data.title;
  description.value = props.photo.data.description;
  date.value = props.photo.date;
  location.value = props.photo.data.location;
  hideThumbnail.value = props.photo.data.hideThumbnail;
  photoPeople.value = props.photo.people;
  camera.value = props.photo.data.camera;
  if (props.photo.data.date.length > 0) {
    focusDate.value = props.photo.date;
  } else {
    focusDate.value = props.prevDate;
  }
  if (props.photo.data.photographer) {
    photographer.value = [props.photo.data.photographer];
  } else {
    photographer.value = [];
  }
}

watch(() => props.photo, initialize);

onMounted(initialize);
</script>

<template>
  <div v-if="photo.data.hideThumbnail && !viewConfirmation">
    <v-btn @click="viewConfirmation = true">Show Image</v-btn>
  </div>
  <div v-if="!photo.data.hideThumbnail || viewConfirmation">
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
      @click="closeUp = true"
    ></v-img>
    <v-img v-if="showRaw" max-height="600" :src="photo.rawFile"></v-img>
    <v-btn v-if="photo.rawFile.length > 0" @click="showRaw = !showRaw">RAW</v-btn>
  </div>
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
    label="Location"
    :items="placeList"
    v-model="location"
    @update:model-value="emit('update:location', location)"
  >
    <template v-slot:item="{ props, item }">
      <v-list-item v-bind="props" :base-color="item.raw.color"></v-list-item>
    </template>
  </v-select>
  <people-input
    label="People"
    :value="photoPeople"
    @update="(people) => emit('update:people', people)"
    multiple
    sort="count"
  ></people-input>
  <v-rating v-model="rating" @update:model-value="emit('update:rating', rating)"></v-rating>
  <people-input
    label="Taken by"
    :value="photographer"
    @update="
      (value) => {
        if (value[0] === undefined) {
          emit('update:photographer', '');
        } else {
          emit('update:photographer', value[0]);
        }
      }
    "
    sort="photographer"
  ></people-input>
  <v-select
    :items="cameraList"
    label="Camera"
    v-model="camera"
    @update:model-value="emit('update:camera', camera)"
  ></v-select>
  <v-text-field
    label="Title"
    v-model="title"
    @update:model-value="emit('update:title', title)"
  ></v-text-field>
  <autosave-text
    label="Description"
    :value="description"
    @save="(description) => emit('update:description', description)"
  ></autosave-text>
  <v-date-input
    label="Date"
    v-model="date"
    :year="focusDate.getFullYear()"
    :month="focusDate.getMonth()"
    @update:model-value="emit('update:date', date.toISOString())"
  ></v-date-input>
  <v-select
    label="Group"
    :items="groupNames"
    v-model="group"
    @update:model-value="emit('update:group', group)"
  ></v-select>
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
  <v-btn
    @click="
      async () => {
        emit('update:location', '');
      }
    "
    >Remove Location</v-btn
  >
  <v-btn
    @click="
      async () => {
        emit('update:date', '');
      }
    "
    >Remove Date</v-btn
  >
  <v-btn
    @click="
      async () => {
        emit('update:rating', '');
      }
    "
    >Remove Rating</v-btn
  >
  <v-btn @click="setPhotoDialog = true">Set As Profile Photo</v-btn>
  <v-checkbox
    label="Hide Thumbnail"
    v-model="hideThumbnail"
    @update:model-value="emit('update:hideThumbnail', hideThumbnail)"
  ></v-checkbox>
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
        <v-img v-if="!photo.data.video" max-height="600" :src="photoPath"></v-img>
      </v-card-text>
    </v-card>
  </v-dialog>
  <v-dialog v-model="setPhotoDialog">
    <v-card>
      <v-card-title>Set Profile Photo</v-card-title>
      <v-card-text>
        Preview:
        <v-avatar size="128">
          <v-img :src="photoPath"></v-img>
        </v-avatar>
        <v-avatar size="48">
          <v-img :src="photoPath"></v-img>
        </v-avatar>
        <br />
        <people-input
          :value="setPhotoTarget"
          label="Set as profile photo for"
          @update="(value: string[]) => (setPhotoTarget = value)"
          sort="count"
        ></people-input>
      </v-card-text>
      <v-card-actions>
        <v-btn @click="setPhotoDialog = false">Cancel</v-btn>
        <v-btn
          color="primary"
          @click="
            async () => {
              await setPersonPhoto(setPhotoTarget[0], photoPath);
              setPhotoDialog = false;
              setPhotoTarget = [];
            }
          "
          >Save</v-btn
        >
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>
