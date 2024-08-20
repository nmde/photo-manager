<script setup lang="ts">
import { VideoPlayer } from '@videojs-player/vue';
import 'video.js/dist/video-js.css';
import { computed, ref } from 'vue';
import { Photo } from '../classes/Photo';
import { fileStore } from '../stores/fileStore';
import type { Person } from '~/classes/Person';

type PeopleEntry = {
  title: string;
  value: string;
  color: string;
};

const {
  groupNames,
  addGroup,
  removeGroup,
  places,
  layers,
  peopleCategories,
  people,
  peopleMap,
  setPersonPhoto,
} = fileStore;

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
const date = ref<Date>(new Date());
const closeUp = ref(false);
const location = ref('');
const showRaw = ref(false);
const photoPeople = ref<PeopleEntry[]>([]);

const setPhotoDialog = ref(false);
const setPhotoTarget = ref('');

const peopleList = computed(() => {
  let flatPeople: Person[] = [];
  Object.values(peopleMap).forEach((persons) => {
    flatPeople = flatPeople.concat(persons);
  });
  return flatPeople
    .sort((a, b) => {
      if (a.count < b.count) {
        return 1;
      }
      if (a.count > b.count) {
        return -1;
      }
      return 0;
    })
    .map((person) => {
      return {
        color: peopleCategories[person.data.category].data.color,
        title: person.data.name,
        value: person.Id,
      };
    });
});

const placeList = computed(() => {
  return Object.values(places)
    .sort((a, b) => {
      if (a.isNewestPlace) {
        return -1;
      }
      if (b.isNewestPlace) {
        return 1;
      }
      if (a.count < b.count) {
        return 1;
      }
      if (a.count > b.count) {
        return -1;
      }
      return 0;
    })
    .map((p) => ({
      color: layers[p.data.layer]?.data.color,
      title: p.data.name,
      value: p.Id,
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
  photoPeople.value = props.photo.people.map((person) => {
    return {
      title: people[person].data.name,
      value: person,
      color: peopleCategories[people[person].data.category].data.color,
    };
  });
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
  <v-img v-if="showRaw" max-height="600" :src="photo.rawFile"></v-img>
  <v-btn v-if="photo.rawFile.length > 0" @click="showRaw = !showRaw">RAW</v-btn>
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
  <v-combobox
    label="People"
    :items="peopleList"
    multiple
    chips
    clearable
    item-value="value"
    v-model="photoPeople"
    @update:model-value="
      () => {
        emit(
          'update:people',
          photoPeople.map((p) => p.value),
        );
      }
    "
  >
    <template v-slot:item="{ item, props }">
      <v-list-item v-bind="props" :style="{ color: item.raw.color }"></v-list-item>
    </template>
    <template v-slot:chip="{ item, props }">
      <v-chip v-bind="props" :color="item.raw.color"></v-chip>
    </template>
  </v-combobox>
  <v-rating v-model="rating" @update:model-value="emit('update:rating', rating)"></v-rating>
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
  <v-date-input
    label="Date"
    v-model="date"
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
  <v-btn @click="setPhotoDialog = true">Set As Profile Photo</v-btn>
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
        Set as profile photo for:
        <v-select :items="peopleList" v-model="setPhotoTarget">
          <template v-slot:item="{ props, item }">
            <v-list-item v-bind="props" :base-color="item.raw.color"></v-list-item>
          </template>
        </v-select>
      </v-card-text>
      <v-card-actions>
        <v-btn @click="setPhotoDialog = false">Cancel</v-btn>
        <v-btn
          color="primary"
          @click="
            async () => {
              await setPersonPhoto(setPhotoTarget, photoPath);
              setPhotoDialog = false;
              setPhotoTarget = '';
            }
          "
          >Save</v-btn
        >
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>
