<script setup lang="ts">
  import type { Layer } from '@/classes/Layer';
  import type { Person } from '@/classes/Person';
  import type { PersonCategory } from '@/classes/PersonCategory';
  import type { Photo } from '@/classes/Photo';
  import type { Place } from '@/classes/Place';
  import { VideoPlayer } from '@videojs-player/vue';
  import { get_people, get_people_categories } from '@/api/people';
  import { get_layers, get_places } from '@/api/places';
  import { useFileStore } from '@/stores/fileStore';
  import 'video.js/dist/video-js.css';

  const store = useFileStore();
  const { reportError, setLastDate } = store;
  const { lastSetDate } = storeToRefs(store);

  const props = defineProps<{
    index: number;
    photos: Photo[];
  }>();

  const emit = defineEmits<{
    (e: 'input-focused', value: boolean): void;
  }>();

  const photo = computed(() => props.photos[props.index] as Photo);

  const photoPath = computed(() =>
    photo.value.thumbnail === undefined ? photo.value?.asset_path : photo.value.thumbnail as string,
  );

  const rating = ref<number>();
  const isDuplicate = ref(false);
  const photoTags = ref<string[]>([]);
  const title = ref<string>();
  const description = ref<string>();
  const date = ref<Date>();
  const closeUp = ref(false);
  const location = ref<string[]>([]);
  const photoPeople = ref<string[]>([]);
  const photographer = ref<string[]>([]);
  const hideThumbnail = ref(false);
  const focusDate = ref(new Date());
  const setPhotoDialog = ref(false);
  const setPhotoTarget = ref<string[]>([]);
  const viewConfirmation = ref(false);
  const placeList = ref<Record<string, Place>>({});
  const layers = ref<Record<string, Layer>>({});
  const people = ref<Record<string, Person>>({});
  const peopleCategories = ref<Record<string, PersonCategory>>({});
  const validTags = ref<string | undefined>();

  async function initialize() {
    viewConfirmation.value = false;
    if (photo.value) {
      rating.value = photo.value.rating ?? undefined;
      isDuplicate.value = photo.value.isDuplicate;
      photoTags.value = photo.value.tags;
      title.value = photo.value.title ?? undefined;
      description.value = photo.value.description ?? undefined;
      date.value = photo.value.date ?? undefined;
      location.value = photo.value.location === null ? [] : [photo.value.location];
      hideThumbnail.value = photo.value.hideThumbnail;
      photoPeople.value = photo.value.people;
      photographer.value = photo.value.photographer === null ? [] : [photo.value.photographer];
      validTags.value = photo.value.valid ? undefined : photo.value.validationMsg;
    }
    await get_people_categories()
      .ok(c => (peopleCategories.value = c))
      .err(msg => reportError(msg))
      .send();
    await get_places()
      .ok(p => (placeList.value = p))
      .err(msg => reportError(msg))
      .send();
    await get_people()
      .ok(p => (people.value = p))
      .err(msg => reportError(msg))
      .send();
    await get_layers()
      .ok(l => (layers.value = l))
      .err(message => reportError(message))
      .send();
  }

  const savingLocation = ref(false);
  async function saveLocation(location: string[]) {
    if (location.length > 0) {
      savingLocation.value = true;
      const l = location[0] as string;
      for (const photo of props.photos) {
        await photo.setLocation(l);
      }
      savingLocation.value = false;
    }
  }

  const savingPeople = ref(false);
  async function savePeople(people: string[]) {
    savingPeople.value = true;
    for (const photo of props.photos) {
      await photo.setPeople(people);
    }
    savingPeople.value = false;
  }

  async function savePhotographer(photographer: string[]) {
    savingPeople.value = true;
    for (const photo of props.photos) {
      await photo.setPhotographer(photographer[0] ?? null);
    }
    savingPeople.value = false;
  }

  const savingTags = ref(false);
  async function saveTags(new_tags: string[]) {
    savingTags.value = true;
    for (const photo of props.photos) {
      await photo.setTags(new_tags);
    }
    validTags.value = props.photos[0]?.valid ? undefined : props.photos[0]?.validationMsg;
    savingTags.value = false;
  }

  const savingDate = ref(false);
  async function saveDate(date: Date) {
    savingDate.value = true;
    for (const photo of props.photos) {
      console.log(date);
      await photo.setDate(date);
    }
    if (date) {
      focusDate.value = date;
      setLastDate(date);
    }
    savingDate.value = false;
  }

  watch(props, initialize);

  onMounted(() => {
    initialize();
    focusDate.value = lastSetDate.value;
  });
</script>

<template>
  <div v-if="photo && photo.hideThumbnail && !viewConfirmation" class="hidden-message">
    This image is hidden.
    <br />
    <v-btn @click="viewConfirmation = true">Show Image</v-btn>
  </div>
  <div v-if="photo" class="photo-detail">
    <div v-if="!photo.hideThumbnail || viewConfirmation">
      <video-player
        v-if="photo.is_video"
        controls
        :height="400"
        :poster="photo.thumbnail as string"
        :src="photo.asset_path"
        :width="700"
      />
      <v-img v-if="!photo.is_video" max-height="600" :src="photoPath" @click="closeUp = true" />
    </div>
    <tag-input
      :id="photo.name"
      filtered
      :label="`Tags (${photo.tags.length})`"
      :loading="savingTags"
      :validation="validTags"
      :value="photoTags"
      @change="tags => saveTags(tags)"
      @focused="val => emit('input-focused', val)"
    />
    <sorted-combo
      :id="photo.name"
      color-key="layer"
      :color-repo="layers"
      :items="placeList"
      label="Location"
      :loading="savingLocation"
      :value="location"
      @focused="val => emit('input-focused', val)"
      @update="location => saveLocation(location)"
    />
    <sorted-combo
      :id="photo.name"
      avatars
      chips
      color-key="category"
      :color-repo="peopleCategories"
      item-size="x-large"
      :items="people"
      label="People"
      :loading="savingPeople"
      multiple
      :value="photoPeople"
      @focused="val => emit('input-focused', val)"
      @update="people => savePeople(people)"
    />
    <v-rating
      v-model="rating"
      clearable
      color="primary"
      @update:model-value="rating => photo.setRating(Number(rating))"
    />
    <sorted-combo
      :id="photo.name"
      avatars
      chips
      color-key="category"
      :color-repo="peopleCategories"
      item-size="x-large"
      :items="people"
      label="Taken By"
      :loading="savingPeople"
      sort-key="photographerCount"
      :value="photographer"
      @focused="val => emit('input-focused', val)"
      @update="people => savePhotographer(people)"
    />
    <v-text-field
      v-model="title"
      clearable
      color="primary"
      label="Title"
      @update:focused="val => emit('input-focused', val)"
      @update:model-value="title => photo.setTitle(title)"
    />
    <autosave-text
      label="Description"
      :value="description ?? ''"
      @focused="val => emit('input-focused', val)"
      @save="description => photo.setDescription(description)"
    />
    <v-date-input
      v-model="date"
      aria-autocomplete="none"
      clearable
      color="primary"
      label="Date"
      :loading="savingDate"
      :month="focusDate.getMonth()"
      :year="focusDate.getFullYear()"
      @update:model-value="date => saveDate(date)"
    />
    <v-checkbox
      v-model="isDuplicate"
      label="Duplicate"
      @update:model-value="isDuplicate => photo.setDuplicate(isDuplicate ?? false)"
    />
    <v-btn @click="setPhotoDialog = true">Set As Profile Photo</v-btn>
    <v-checkbox
      v-model="hideThumbnail"
      label="Hide Thumbnail"
      @update:model-value="hideThumbnail => photo.setHideThumbnail(hideThumbnail ?? false)"
    />
  </div>
  <v-dialog v-model="closeUp">
    <v-card :title="photo.name">
      <v-card-text>
        <video-player
          v-if="photo.is_video"
          controls
          :height="400"
          :poster="photo.thumbnail as string"
          :src="photo.asset_path"
          :width="700"
        />
        <v-img v-if="!photo.is_video" max-height="calc(100vh - 136px)" :src="photoPath" />
      </v-card-text>
    </v-card>
  </v-dialog>
  <v-dialog v-model="setPhotoDialog">
    <v-card title="Set Profile Photo">
      <v-card-text>
        Preview:
        <v-avatar size="128">
          <v-img :src="photoPath" />
        </v-avatar>
        <v-avatar size="48">
          <v-img :src="photoPath" />
        </v-avatar>
        <br />
        <sorted-combo
          :id="photo.name"
          avatars
          chips
          color-key="category"
          :color-repo="peopleCategories"
          item-size="x-large"
          :items="people"
          label="Profile Photo"
          :value="setPhotoTarget"
          @focused="val => emit('input-focused', val)"
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
  .photo-detail {
    max-height: calc(100vh - 64px);
    overflow-y: scroll;
  }

  .hidden-message {
    padding: 16px;
  }
</style>
