<script setup lang="ts">
  import type { LayerRec } from '@/classes/Layer';
  import type { Photo } from '@/classes/Photo';
  import type { Place, PlaceRec } from '@/classes/Place';
  import { photo_grid } from '@/api/app';
  import { get_layers, get_places } from '@/api/places';
  import { useFileStore } from '@/stores/fileStore';

  const store = useFileStore();
  const router = useRouter();

  const jumpDate = ref<Date>(new Date());
  const viewDate = ref(new Date());
  const dayDialog = ref(false);
  const dialogDate = ref<string>('');
  const places = ref<PlaceRec>({});
  const layers = ref<LayerRec>({});
  const photos = ref<Photo[]>([]);

  const { reportError, setCalendarViewDate } = store;
  const { calendarViewDate } = storeToRefs(store);

  function dateWithoutTimezone(value: string) {
    const d = new Date(value);
    const userTimezoneOffset = d.getTimezoneOffset() * 60_000;
    return new Date(d.getTime() + userTimezoneOffset);
  }

  const MAX_PHOTOS = 20;

  function getPhotosByDate(date: string) {
    const d = dateWithoutTimezone(date);
    const filtered = photos.value.filter(
      photo =>
        photo.date !== null
        && photo.date.getFullYear() === d.getFullYear()
        && photo.date.getMonth() === d.getMonth()
        && photo.date.getDate() === d.getDate(),
    );
    // Bin by rating to show random stuff still sorted by rating
    const bins: [Photo[], Photo[], Photo[], Photo[], Photo[], Photo[]] = [[], [], [], [], [], []];
    for (const photo of filtered) {
      bins[photo.rating ?? 0]?.push(photo as Photo);
    }
    const sorted: Photo[] = [];
    let bin = 5;
    while (sorted.length < MAX_PHOTOS && bin >= 0) {
      while (bins[bin]?.length === 0) {
        bin -= 1;
      }
      const idx = Math.floor(Math.random() * (bins[bin]?.length ?? 0));
      const p = bins[bin]?.splice(idx, 1)?.[0];
      if (p !== undefined) {
        sorted.push(p);
      }
    }
    return sorted;
  }

  function getLocationsByDate(date: string) {
    const locations: Place[] = [];
    for (const photo of getPhotosByDate(date)) {
      if (photo.location !== null) {
        const place = places.value[photo.location];
        if (place && !locations.some(p => p.id === place.id)) {
          locations.push(place);
        }
      }
    }
    return locations;
  }

  function getAvgRatingByDate(date: string) {
    let sum = 0;
    let count = 0;
    for (const photo of getPhotosByDate(date)) {
      if (photo.rating) {
        sum += photo.rating;
        count += 1;
      }
    }
    return sum / count;
  }

  async function setDate(date: Date) {
    viewDate.value = date;
    setCalendarViewDate(date);
    const year = date.getFullYear();
    const month = date.getMonth() + 1;
    await photo_grid(
      [
        'has:date',
        `date>=${year}-${month.toString().padStart(2, '0')}-01`,
        `date<=${year}-${month.toString().padStart(2, '0')}-${new Date(year, month, 0).getDate()}`,
      ],
      'rating_desc',
    )
      .ok(p => (photos.value = p))
      .err(reportError)
      .send();
  }

  onMounted(async () => {
    await setDate(calendarViewDate.value);
    await get_places()
      .ok(p => (places.value = p))
      .err(reportError)
      .send();
    await get_layers()
      .ok(l => (layers.value = l))
      .err(reportError)
      .send();
  });
</script>

<template>
  <div class="calendar-page">
    <v-toolbar color="primary">
      <div class="toolbar-controls">
        <v-date-input
          v-model="jumpDate"
          label="Jump to"
          @update:model-value="async () => await setDate(jumpDate)"
        />
      </div>
      <template #extension>
        <v-btn class="me-4" variant="outlined"> Today </v-btn>
        <v-btn
          icon
          @click="
            () => {
              setDate(new Date(viewDate.getFullYear(), viewDate.getMonth() - 1));
            }
          "
        >
          <v-icon>mdi-arrow-left</v-icon>
        </v-btn>
        <v-btn
          icon
          @click="
            () => {
              setDate(new Date(viewDate.getFullYear(), viewDate.getMonth() + 1));
            }
          "
        >
          <v-icon>mdi-arrow-right</v-icon>
        </v-btn>
        <v-toolbar-title>
          {{
            [
              'January',
              'February',
              'March',
              'April',
              'May',
              'June',
              'July',
              'August',
              'September',
              'October',
              'November',
              'December',
            ][viewDate.getMonth()]
          }}
          {{ viewDate.getFullYear() }}
        </v-toolbar-title>
      </template>
    </v-toolbar>
    <div class="calendar">
      <v-calendar
        v-model="viewDate"
        hide-week-number
        type="month"
        @update:model-value="async () => await setDate(viewDate)"
      >
        <template #day="{ date }">
          <div class="calendar-photos">
            <photo-icon
              v-for="photo in getPhotosByDate(date).slice(0, 2)"
              :key="photo.name"
              hide-icons
              :photo="photo as Photo"
              :size="100"
              @select="
                () => {
                  dialogDate = date;
                  dayDialog = true;
                }
              "
            />
          </div>
        </template>
      </v-calendar>
    </div>
    <v-dialog v-model="dayDialog">
      <v-card :title="dialogDate">
        <v-card-text>
          <v-container>
            <v-row>
              <v-col class="calendar-photos" cols="6">
                <photo-icon
                  v-for="photo in getPhotosByDate(dialogDate).slice(0, MAX_PHOTOS)"
                  :key="photo.name"
                  hide-icons
                  :photo="photo as Photo"
                  :size="100"
                  @select="
                    () => {
                      router.push(`/tagger?date=${dialogDate}`);
                    }
                  "
                />
              </v-col>
              <v-col cols="6">
                <div v-if="getPhotosByDate(dialogDate).length > 0">
                  <br />
                  Total photos: {{ getPhotosByDate(dialogDate).length }}
                  <br />
                  Average rating: {{ getAvgRatingByDate(dialogDate) }}
                </div>
                <v-chip
                  v-for="place in getLocationsByDate(dialogDate)"
                  :key="place.id"
                  :color="layers[place.layer]?.color"
                >
                  {{ place.name }}
                </v-chip>
              </v-col>
            </v-row>
          </v-container>
        </v-card-text>
      </v-card>
    </v-dialog>
  </div>
</template>

<style scoped>
  .calendar-page {
    height: 100vh;
    overflow-y: scroll;
  }

  .calendar {
    margin: 16px;
  }

  .calendar-photos {
    display: flex;
    flex-wrap: wrap;
  }

  .toolbar-controls {
    margin-left: 18px;
  }
</style>
