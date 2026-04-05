<script setup lang="ts">
  import type { Layer } from '@/classes/Layer';
  import type { Photo } from '@/classes/Photo';
  import type { Place } from '@/classes/Place';
  import { photo_grid } from '@/api/photos';
  import { get_layers, get_places } from '@/api/places';
  import { useFileStore } from '@/stores/fileStore';

  const store = useFileStore();
  const router = useRouter();

  const jumpDate = ref<Date>(new Date());
  const date = ref(new Date());
  const dayDialog = ref(false);
  const dialogDate = ref<Date>(new Date());
  const places = ref<Record<string, Place>>({});
  const layers = ref<Record<string, Layer>>({});

  const { calendarViewDate, setCalendarViewDate } = store;

  type Event = {
    start: Date;
    end: Date;
    allDay: boolean;
    photos: Photo[];
  };

  const events = ref<Event[]>([]);

  let eventMap: Record<
    string,
    {
      date: Date;
      photos: Photo[];
    }
  > = {};
  async function buildEventMap() {
    eventMap = {};
    if (date.value) {
      const year = date.value.getFullYear();
      const month = date.value.getMonth() + 1;
      for (const photo of await photo_grid(
        [
          'has:date',
          `date>=${year}-${month.toString().padStart(2, '0')}-01`,
          `date<=${year}-${month.toString().padStart(2, '0')}-${new Date(
            year,
            month,
            0,
          ).getDate()}`,
        ],
        'ratingdesc',
      )) {
        const k = photo.date?.toISOString();
        if (k) {
          if (!eventMap[k]) {
            eventMap[k] = { date: photo.date as Date, photos: [] };
          }
          eventMap[k].photos.push(photo);
        }
      }
      for (let i = 1; i <= new Date(year, month, 0).getDate(); i += 1) {
        const d = new Date(year, month, i);
        const k = d.toISOString();
        if (!eventMap[k]) {
          eventMap[k] = {
            date: d,
            photos: [],
          };
        }
      }
    }
    events.value = Object.values(eventMap).map(ev => ({
      start: ev.date,
      end: ev.date,
      allDay: true,
      photos: ev.photos,
    }));
  }

  function getLocationsByDate(date: Date) {
    const locations: Place[] = [];
    const photos = eventMap[date.toISOString()]?.photos;
    if (photos) {
      for (const photo of photos) {
        if (photo.location !== undefined) {
          const place = places.value[photo.location];
          if (place && !locations.some(p => p.id === place.id)) {
            locations.push(place);
          }
        }
      }
    }
    return locations;
  }

  function getAvgRatingByDate(date: Date) {
    let sum = 0;
    let count = 0;
    const photos = eventMap[date.toISOString()]?.photos;
    if (photos) {
      for (const photo of photos) {
        if (photo.rating) {
          sum += photo.rating;
          count += 1;
        }
      }
    }
    return sum / count;
  }

  onMounted(async () => {
    date.value = calendarViewDate;
    await buildEventMap();
    places.value = await get_places();
    layers.value = await get_layers();
  });
</script>

<template>
  <div class="calendar-page">
    <v-toolbar color="primary">
      <div class="toolbar-controls">
        <v-date-input
          v-model="jumpDate"
          color="primary"
          label="Jump to"
          @update:model-value="
            async () => {
              date = jumpDate;
              setCalendarViewDate(date);
              await buildEventMap();
            }
          "
        />
      </div>
    </v-toolbar>
    <div class="calendar">
      <v-calendar
        v-model="date"
        :events="events"
        hide-week-number
        type="month"
        @update:model-value="
          async () => {
            setCalendarViewDate(date);
            await buildEventMap();
          }
        "
      >
        <template #event="{ day, event }">
          <div class="calendar-photos">
            <photo-icon
              v-for="photo in (event as Event)?.photos.slice(0, 4)"
              :key="photo.name"
              hide-icons
              :photo="photo"
              :size="100"
              @select="
                () => {
                  dialogDate = new Date(day?.date);
                  dayDialog = true;
                }
              "
            />
          </div>
        </template>
      </v-calendar>
    </div>
    <v-dialog v-model="dayDialog">
      <v-card :title="dialogDate.toDateString()">
        <v-card-text>
          <v-container>
            <v-row>
              <v-col class="calendar-photos" cols="6">
                <photo-icon
                  v-for="photo in eventMap[dialogDate.toISOString()]?.photos.slice(0, 20)"
                  :key="photo.name"
                  hide-icons
                  :photo="photo"
                  :size="100"
                  @select="
                    () => {
                      router.push(`/tagger?date=${dialogDate.toISOString()}`);
                    }
                  "
                />
              </v-col>
              <v-col cols="6">
                <div v-if="(eventMap[dialogDate.toISOString()]?.photos.length ?? 0) > 0">
                  <br>
                  Total photos: {{ eventMap[dialogDate.toISOString()]?.photos.length }}
                  <br>
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
    height: 100%;
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
