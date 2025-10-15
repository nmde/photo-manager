<script setup lang="ts">
  import type { Photo } from '../classes/Photo';
  import type { Place } from '../classes/Place';
  import { computed, onMounted, ref } from 'vue';
  import { useRouter } from 'vue-router';
  import { fileStore, formatDate, moods } from '../stores/fileStore';

  const router = useRouter();

  const jumpDate = ref<Date>(new Date());
  const date = ref<Date[]>([new Date()]);
  const dayDialog = ref(false);
  const dialogDate = ref<Date>(new Date());

  const { files, calendarViewDate, setCalendarViewDate, places, layers, journals } = fileStore;

  let eventMap: Record<
    string,
    {
      date: Date;
      photos: Photo[];
    }
  > = {};
  function buildEventMap() {
    eventMap = {};
    const groups: string[] = [];
    for (const photo of Object.values(files)) {
      if (
        photo.hasDate &&
        (photo.group === undefined || (photo.group && !groups.includes(photo.group)))
      ) {
        const k = photo.date.toISOString();
        if (!eventMap[k]) {
          eventMap[k] = { date: photo.date, photos: [] };
        }
        eventMap[k].photos.push(photo);
        eventMap[k].photos.sort((a, b) => {
          if (!b.rating) {
            return -1;
          }
          if (!a.rating) {
            return 1;
          }
          if (a.rating > b.rating) {
            return -1;
          }
          if (a.rating < b.rating) {
            return 1;
          }
          return 0;
        });
        if (photo.group !== undefined) {
          groups.push(photo.group);
        }
      }
    }
    const year = date.value[0]?.getFullYear();
    const month = date.value[0]?.getMonth();
    if (year && month) {
      for (let i = 1; i <= new Date(year, month + 1, 0).getDate(); i += 1) {
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
  }

  type Event = {
    start: Date;
    end: Date;
    allDay: boolean;
    photos: Photo[];
  };

  const events = computed(() => {
    const events: Event[] = [];
    buildEventMap();
    for (const event of Object.values(eventMap)) {
      events.push({
        start: event.date,
        end: event.date,
        allDay: true,
        photos: event.photos,
      });
    }
    return events;
  });

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

  onMounted(() => {
    console.log(calendarViewDate);
    date.value[0] = calendarViewDate;
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
              () => {
                date[0] = jumpDate;
                setCalendarViewDate(date[0]);
              }
            "
          />
          <v-calendar
            v-model="date"
            :events="events"
            hide-week-number
            type="month"
            @update:model-value="
              () => {
                const d = date[0];
                if (d) {
                  setCalendarViewDate(d);
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
              <v-col cols="6">
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
                <v-chip
                  v-for="place in getLocationsByDate(dialogDate)"
                  :key="place.id"
                  :color="layers[place.layer]?.color"
                >
                  {{ place.name }}
                </v-chip>
                <div v-if="(eventMap[dialogDate.toISOString()]?.photos.length ?? 0) > 0">
                  <br>
                  Total photos: {{ eventMap[dialogDate.toISOString()]?.photos.length }}
                  <br>
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
</style>
