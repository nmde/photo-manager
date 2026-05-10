<script setup lang="ts">
  import type { LayerRec } from '@/classes/Layer';
  import type { PersonRec } from '@/classes/Person';
  import type { PersonCategoryRec } from '@/classes/PersonCategory';
  import type { Photo, PhotoData } from '@/classes/Photo';
  import type { PlaceRec } from '@/classes/Place';
  import { openPath, revealItemInDir } from '@tauri-apps/plugin-opener';
  import { VideoPlayer } from '@videojs-player/vue';
  import { useRules } from 'vuetify/labs/rules';
  import { get_people, get_people_categories } from '@/api/people';
  import { get_grouped_raw } from '@/api/photos';
  import { get_layers, get_places } from '@/api/places';
  import { useFileStore } from '@/stores/fileStore';
  import 'video.js/dist/video-js.css';

  const router = useRouter();
  const rules = useRules();
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
    photo.value.thumbnail === null ? photo.value.asset_path : (photo.value.thumbnail as string),
  );

  const rating = ref<number>();
  const isDuplicate = ref<PhotoData['is_duplicate']>();
  const photoTags = ref<PhotoData['tags']>([]);
  const title = ref<PhotoData['title']>();
  const description = ref<PhotoData['description']>();
  const date = ref<Date>();
  const closeUp = ref(false);
  const location = ref<string[]>([]);
  const photoPeople = ref<PhotoData['people']>([]);
  const photographer = ref<string[]>([]);
  const hideThumbnail = ref<PhotoData['hide_thumbnail']>();
  const focusDate = ref(new Date());
  const setPhotoDialog = ref(false);
  const setPhotoTarget = ref<string[]>([]);
  const viewConfirmation = ref(false);
  const placeList = ref<PlaceRec>({});
  const layers = ref<LayerRec>({});
  const people = ref<PersonRec>({});
  const peopleCategories = ref<PersonCategoryRec>({});
  const validTags = ref<string | undefined>();
  const showRaw = ref(false);
  const rawLoading = ref(false);
  const rawThumbnail = ref<string>();

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
      validTags.value = photo.value.valid_tags.is_valid
        ? undefined
        : (photo.value.valid_tags.message ?? undefined);
    }
    await get_people_categories()
      .ok(c => (peopleCategories.value = c))
      .err(reportError)
      .send();
    await get_places()
      .ok(p => (placeList.value = p))
      .err(reportError)
      .send();
    await get_people()
      .ok(p => (people.value = p))
      .err(reportError)
      .send();
    await get_layers()
      .ok(l => (layers.value = l))
      .err(reportError)
      .send();
  }

  const savingLocation = ref(false);
  async function saveLocation(location: string[]) {
    savingLocation.value = true;
    const l = location[0] as string;
    for (const photo of props.photos) {
      await photo.setLocation(l);
    }
    savingLocation.value = false;
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
    validTags.value = props.photos[0]?.valid_tags.is_valid
      ? undefined
      : (props.photos[0]?.valid_tags.message ?? undefined);
    savingTags.value = false;
  }

  const savingDate = ref(false);
  async function saveDate(date: Date) {
    savingDate.value = true;
    for (const photo of props.photos) {
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
      <v-img v-else max-height="600" :src="photoPath" @click="closeUp = true" />
    </div>
    <template v-if="photo.grouped_raw !== null">
      <v-btn
        block
        class="show-raw"
        :loading="rawLoading"
        @click="
          async () => {
            showRaw = !showRaw;
            if (rawThumbnail === undefined && photo.grouped_raw !== null) {
              rawLoading = true;
              rawThumbnail = (await get_grouped_raw(photo.grouped_raw)) ?? undefined;
              rawLoading = false;
            }
          }
        "
      >
        {{ showRaw ? 'Hide' : 'Show' }} RAW
      </v-btn>
      <v-img v-if="showRaw && rawThumbnail !== undefined" max-height="600" :src="rawThumbnail" />
    </template>
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
    <v-alert
      v-if="photo.metadata_location !== null"
      :type="photo.location === null ? 'info' : undefined"
    >
      File location: {{ photo.metadata_location.join(', ') }}
      <v-btn
        color="secondary"
        density="comfortable"
        @click="
          router.push(`/locations?center=${encodeURIComponent(photo.metadata_location.join(','))}`)
        "
      >
        Show On Map
      </v-btn>
    </v-alert>
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
      sort-key="photographer_count"
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
    <v-container>
      <v-row>
        <v-col>
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
          >
            <template #append>
              <v-menu>
                <template #activator="{ props: vprops }">
                  <v-btn v-bind="vprops" icon variant="flat">
                    <v-icon>mdi-dots-vertical</v-icon>
                  </v-btn>
                </template>
                <v-list>
                  <v-list-item>Search This Date</v-list-item>
                  <v-list-item>Show In Calendar</v-list-item>
                </v-list>
              </v-menu>
            </template>
          </v-date-input>
        </v-col>
      </v-row>
    </v-container>
    <v-alert v-if="photo.metaDate !== null" :type="photo.date === null ? 'info' : undefined">
      File date: {{ photo.metaDate }}
      <v-btn
        color="primary"
        density="comfortable"
        @click="
          () => {
            if (photo.metaDate !== null) {
              date = photo.metaDate;
              saveDate(photo.metaDate);
            }
          }
        "
      >
        Set As Date
      </v-btn>
    </v-alert>
    <v-checkbox
      v-model="isDuplicate"
      label="Duplicate"
      @update:model-value="isDuplicate => photo.setDuplicate(isDuplicate ?? false)"
    />
    <v-checkbox
      v-model="hideThumbnail"
      label="Hide Thumbnail"
      @update:model-value="hideThumbnail => photo.setHideThumbnail(hideThumbnail ?? false)"
    />
    <v-btn @click="setPhotoDialog = true">Set As Profile Photo</v-btn>
    <v-btn
      color="primary"
      @click="
        async () => {
          await openPath(photo.name);
        }
      "
    >
      Open File
    </v-btn>
    <v-btn
      color="primary"
      @click="
        async () => {
          await revealItemInDir(photo.name);
        }
      "
    >
      Reveal in Explorer
    </v-btn>
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
  <form-dialog
    v-model="setPhotoDialog"
    :reset="() => (setPhotoTarget = [])"
    title="Set As Profile Photo"
    @submit="
      async () => {
        if (setPhotoTarget[0]) {
          await people[setPhotoTarget[0]]?.setPhoto(photoPath);
        }
      }
    "
  >
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
      :rules="[rules.required()]"
      :value="setPhotoTarget"
      @focused="val => emit('input-focused', val)"
      @update="value => (setPhotoTarget = value)"
    />
  </form-dialog>
</template>

<style scoped>
  .photo-detail {
    max-height: calc(100vh - 128px);
    overflow-y: scroll;
    padding: 0 var(--space-md) var(--space-md);
  }

  .hidden-message {
    padding: var(--space-md);
  }

  .show-raw {
    padding: var(--space-md);
  }
</style>
