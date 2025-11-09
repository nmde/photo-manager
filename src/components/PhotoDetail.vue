<script setup lang="ts">
  import type { Photo } from '../classes/Photo';
  import { invoke } from '@tauri-apps/api/core';
  import { VideoPlayer } from '@videojs-player/vue';
  import { Layer, type LayerData } from '@/classes/Layer';
  import { Place, type PlaceData } from '@/classes/Place';
  import { fileStore } from '../stores/fileStore';
  import AutosaveText from './AutosaveText.vue';
  import PeopleInput from './PeopleInput.vue';
  import TagInput from './TagInput.vue';
  import 'video.js/dist/video-js.css';

  const { groupNames, addGroup } = fileStore;

  const emit = defineEmits<{
    (
      e:
        | 'update:title'
        | 'update:description'
        | 'update:date'
        | 'update:location'
        | 'update:photographer'
        | 'update:camera',
      value: string,
    ): void;
    (e: 'update:tags' | 'update:people', value: string[]): void;
    (e: 'update:rating', rating: number): void;
    (e: 'update:isDuplicate' | 'update:hideThumbnail', value: boolean): void;
    (e: 'update:group', group?: string): void;
  }>();

  const props = defineProps<{
    photo: Photo;
    prevDate: Date;
    loading?: boolean;
  }>();

  const photoPath = computed(() =>
    props.photo.thumbnail.length > 0 ? props.photo.thumbnail : props.photo.path,
  );

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
  const placeList = ref<Place[]>([]);
  const layers = ref<Record<string, Layer>>({});

  const cameraList = computed(() => []);

  async function initialize() {
    rating.value = props.photo.rating ?? 0;
    isDuplicate.value = props.photo.isDuplicate;
    group.value = props.photo.group;
    photoTags.value = props.photo.tags;
    title.value = props.photo.title;
    description.value = props.photo.description;
    date.value = props.photo.hasDate ? props.photo.date : new Date();
    location.value = props.photo.location;
    hideThumbnail.value = props.photo.hideThumbnail;
    photoPeople.value = props.photo.people;
    camera.value = props.photo.camera;
    focusDate.value = props.photo.hasDate ? props.photo.date : props.prevDate;
    photographer.value = props.photo.photographer ? [props.photo.photographer] : [];
    placeList.value = Object.values(
      Place.createPlaces(await invoke<Record<string, PlaceData>>('get_places')),
    ).toSorted((a, b) => b.count - a.count);
    layers.value = Layer.createLayers(await invoke<LayerData[]>('get_layers'));
  }

  watch(() => props.photo, initialize);

  onMounted(initialize);
</script>

<template>
  <div v-if="photo.hideThumbnail && !viewConfirmation">
    <v-btn @click="viewConfirmation = true">Show Image</v-btn>
  </div>
  <div v-if="!photo.hideThumbnail || viewConfirmation">
    <video-player
      v-if="photo.video"
      controls
      :height="400"
      :poster="photo.thumbnail"
      :src="photo.path"
      :width="700"
    />
    <v-img v-if="!photo.video" max-height="600" :src="photoPath" @click="closeUp = true" />
    <v-img v-if="showRaw" max-height="600" :src="photo.rawFile" />
    <v-btn v-if="photo.rawFile.length > 0" @click="showRaw = !showRaw">RAW</v-btn>
  </div>
  <tag-input
    advanced
    filtered
    :label="`Photo Tags (${photo.tags.length})`"
    :loading="loading"
    :target="photo"
    validate
    :value="photo.tags"
    @change="
      tags => {
        emit('update:tags', tags);
      }
    "
  />
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
    v-model="location"
    clearable
    item-title="name"
    item-value="id"
    :items="placeList"
    label="Location"
    @update:model-value="
      async (location: string | null) => {
        emit('update:location', location ?? '');
        placeList = Object.values(
          Place.createPlaces(await invoke<Record<string, PlaceData>>('get_places')),
        ).toSorted((a, b) => b.count - a.count);
      }
    "
  >
    <template #item="{ props: lprops, item }">
      <v-list-item
        v-bind="lprops"
        :base-color="layers[item.raw.layer]?.color"
        :title="`${item.title} (${item.raw.count})`"
      />
    </template>
  </v-select>
  <people-input
    label="People"
    multiple
    sort="count"
    :value="photoPeople"
    @update="people => emit('update:people', people)"
  />
  <div class="input-group">
    <v-rating
      v-model="rating"
      @update:model-value="rating => emit('update:rating', Number(rating))"
    />
    <v-btn icon @click="emit('update:rating', 0)">
      <v-icon>mdi-close</v-icon>
    </v-btn>
  </div>
  <people-input
    label="Taken by"
    sort="photographer"
    :value="[photographer]"
    @update="
      value => {
        if (value[0] === undefined) {
          emit('update:photographer', '');
        } else {
          emit('update:photographer', value[0]);
        }
      }
    "
  />
  <v-select
    v-model="camera"
    :items="cameraList"
    label="Camera"
    @update:model-value="camera => emit('update:camera', camera)"
  />
  <v-text-field
    v-model="title"
    label="Title"
    @update:model-value="title => emit('update:title', title)"
  />
  <autosave-text
    label="Description"
    :value="description"
    @save="description => emit('update:description', description)"
  />
  <v-date-input
    v-model="date"
    label="Date"
    @update:model-value="date => emit('update:date', date.toISOString())"
  />
  <v-select
    v-model="group"
    :items="groupNames"
    label="Group"
    @update:model-value="group => emit('update:group', group)"
  />
  <v-checkbox
    v-model="isDuplicate"
    label="Mark as duplicate"
    @update:model-value="isDuplicate => emit('update:isDuplicate', isDuplicate ?? false)"
  />
  <v-btn icon @click="showAddGroup = !showAddGroup">
    <v-icon>mdi-plus</v-icon>
  </v-btn>
  <v-btn
    icon
    @click="
      async () =>
        await invoke('set_photo_group', {
          photo: photo.id,
          value: '',
        })
    "
  >
    <v-icon>mdi-trash-can</v-icon>
  </v-btn>
  <v-btn @click="emit('update:date', '')"> Remove Date </v-btn>
  <v-btn @click="setPhotoDialog = true">Set As Profile Photo</v-btn>
  <v-checkbox
    v-model="hideThumbnail"
    label="Hide Thumbnail"
    @update:model-value="hideThumbnail => emit('update:hideThumbnail', hideThumbnail ?? false)"
  />
  <div v-if="showAddGroup">
    <v-text-field v-model="newGroupName" label="New Group Name" />
    <v-btn
      color="primary"
      :error="newGroupError"
      @click="
        () => {
          if (groupNames.includes(newGroupName)) {
            newGroupError = true;
          } else if (newGroupName.length > 0) {
            addGroup(newGroupName);
            newGroupName = '';
            newGroupError = false;
          }
        }
      "
    >
      Create Group
    </v-btn>
  </div>
  <v-dialog v-model="closeUp">
    <v-card>
      <v-card-title>{{ photo.title }}</v-card-title>
      <v-card-text>
        <video-player
          v-if="photo.video"
          controls
          :height="400"
          :poster="photo.thumbnail"
          :src="photo.path"
          :width="700"
        />
        <v-img v-if="!photo.video" max-height="600" :src="photoPath" />
      </v-card-text>
    </v-card>
  </v-dialog>
  <v-dialog v-model="setPhotoDialog">
    <v-card>
      <v-card-title>Set Profile Photo</v-card-title>
      <v-card-text>
        Preview:
        <v-avatar size="128">
          <v-img :src="photoPath" />
        </v-avatar>
        <v-avatar size="48">
          <v-img :src="photoPath" />
        </v-avatar>
        <br />
        <people-input
          label="Set as profile photo for"
          sort="count"
          :value="setPhotoTarget"
          @update="value => (setPhotoTarget = value)"
        />
      </v-card-text>
      <v-card-actions>
        <v-btn @click="setPhotoDialog = false">Cancel</v-btn>
        <v-btn
          color="primary"
          @click="
            async () => {
              if (setPhotoTarget[0]) {
                await people[setPhotoTarget[0]]?.setPhoto(photoPath);
              }
              setPhotoDialog = false;
              setPhotoTarget = [];
            }
          "
        >
          Save
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<style scoped>
  .input-group {
    display: flex;
    margin-bottom: 8px;
  }
</style>
