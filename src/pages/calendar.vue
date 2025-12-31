<script setup lang="ts">
  import type { Place } from '../classes/Place';
  import { invoke } from '@tauri-apps/api/core';
  import { useRouter } from 'vue-router';
  import { Photo, type PhotoData } from '../classes/Photo';
  import { fileStore, formatDate, moods } from '../stores/fileStore';

  const router = useRouter();

  const jumpDate = ref<Date>(new Date());
  const date = ref<Date[]>([new Date()]);
  const dayDialog = ref(false);
  const dialogDate = ref<Date>(new Date());

  const { calendarViewDate, setCalendarViewDate, journals } = fileStore;

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
    if (date.value[0]) {
      const year = date.value[0].getFullYear();
      const month = date.value[0].getMonth() + 1;
      for (const photo of Photo.createPhotos(
        (
          await invoke<{ photos: PhotoData[] }>('photo_grid', {
            query: [
              'has:date',
              `date>=${year}-${month.toString().padStart(2, '0')}-01`,
              `date<=${year}-${month.toString().padStart(2, '0')}-${new Date(
                year,
                month,
                0,
              ).getDate()}`,
            ],
            sort: 'rating_desc',
          })
        ).photos,
      )) {
        console.log(photo.date);
        const k = photo.date.toISOString();
        if (!eventMap[k]) {
          eventMap[k] = { date: photo.date, photos: [] };
        }
        eventMap[k].photos.push(photo);
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
    console.log(eventMap);
  }

  function getLocationsByDate(date: Date) {
    const locations: Place[] = [];
    const photos = eventMap[date.toISOString()]?.photos;
    if (photos) {
      for (const photo of photos) {
        if (photo.hasLocation) {
          const place = places[photo.location];
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
    date.value[0] = calendarViewDate;
    await buildEventMap();
  });
</script>

<template>
  <v-main class="main">
    <v-container fluid>
      <v-row>
        <v-col>
          <v-date-input
            v-model="jumpDate"
            label="Jump to"
            @update:model-value="
              async () => {
                date[0] = jumpDate;
                setCalendarViewDate(date[0]);
                await buildEventMap();
              }
            "
          />
          <v-calendar
            v-model="date"
            :events="events"
            hide-week-number
            type="month"
            @update:model-value="
              async () => {
                const d = date[0];
                if (d) {
                  setCalendarViewDate(d);
                  await buildEventMap();
                  console.log(events);
                }
              }
            "
          >
            <template #day-event="{ day, event }">
              <div
                v-if="journals[formatDate(day?.date ?? new Date())]"
                :class="{ 'event-bg': true, 'event-bg-half': ((event as Event).photos?.length ?? 0) < 3 }"
                :style="{
                  backgroundColor:
                    moods[journals[formatDate(day?.date ?? new Date())]?.mood ?? 0]?.color,
                }"
                @click="
                  () => {
                    dialogDate = day?.date ?? new Date();
                    dayDialog = true;
                  }
                "
              />
              <div class="calendar-photos">
                <photo-icon
                  v-for="photo in (event as Event)?.photos.slice(0, 4)"
                  :key="photo.id"
                  hide-icons
                  :photo="photo"
                  :size="100"
                  @select="
                    () => {
                      dialogDate = day?.date ?? new Date();
                      dayDialog = true;
                    }
                  "
                />
              </div>
              <div
                v-if="(event as Event).photos.length === 0 && !journals[formatDate(day?.date ?? new Date())]"
                class="focus"
                @click="
                  () => {
                    dialogDate = day?.date ?? new Date();
                    dayDialog = true;
                  }
                "
              />
            </template>
          </v-calendar>
        </v-col>
      </v-row>
    </v-container>
    <v-dialog v-model="dayDialog">
      <v-card>
        <v-card-title>
          {{ dialogDate.toDateString() }}
          <mood-icon
            v-if="journals[formatDate(dialogDate)]"
            :mood="journals[formatDate(dialogDate)]?.mood ?? 0"
          />
        </v-card-title>
        <v-card-text>
          <v-container>
            <v-row>
              <v-col class="calendar-photos" cols="6">
                <photo-icon
                  v-for="photo in eventMap[dialogDate.toISOString()]?.photos.slice(0, 20)"
                  :key="photo.id"
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
                  <br />
                  Total photos: {{ eventMap[dialogDate.toISOString()]?.photos.length }}
                  <br />
                  Average rating: {{ getAvgRatingByDate(dialogDate) }}
                </div>
              </v-col>
            </v-row>
          </v-container>
        </v-card-text>
        <v-card-actions>
          <v-btn
            color="primary"
            @click="
              () => {
                router.push(`/journal?date=${dialogDate.toISOString()}`);
              }
            "
          >
            Open In Journal
          </v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>
  </v-main>
</template>

<style scoped>
  .focus {
    height: 200px;
    width: 200px;
    display: block;
  }

  .event-bg {
    width: 100%;
    height: 214px;
    position: absolute;
    opacity: 0.5;
  }

  .event-bg-half {
    height: 118px;
  }

  .calendar-photos {
    display: flex;
    flex-wrap: wrap;
  }
</style>
