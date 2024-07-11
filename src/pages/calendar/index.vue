<script setup lang="ts">
import { computed } from 'vue';
import { fileStore } from '../../stores/fileStore';
import { Photo } from '../../classes/Photo';
import type { Place } from '~/classes/Place';

const router = useRouter();

const jumpDate = ref<Date>(new Date());
const date = ref<Date[]>([new Date()]);
const dayDialog = ref(false);
const dialogDate = ref<Date>(new Date());

const { files, calendarViewDate, setCalendarViewDate, places, layers } = fileStore;

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
  Object.values(files).forEach((photo) => {
    if (photo.data.date.length > 0) {
      if (photo.group === undefined || groups.indexOf(photo.group) < 0) {
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
  });
  const year = date.value[0].getFullYear();
  const month = date.value[0].getMonth();
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

const events = computed(() => {
  const events: any[] = [];
  buildEventMap();
  Object.values(eventMap).forEach((event) => {
    events.push({
      start: event.date,
      end: event.date,
      allDay: true,
      photos: event.photos,
    });
  });
  return events;
});

function getLocationsByDate(date: Date) {
  const locations: Place[] = [];
  eventMap[date.toISOString()].photos.forEach((photo) => {
    if (photo.hasLocation) {
      const place = places[photo.data.location];
      if (place) {
        if (locations.findIndex((p) => p.Id === place.Id) < 0) {
          locations.push(place);
        }
      }
    }
  });
  return locations;
}

function getAvgRatingByDate(date: Date) {
  let sum = 0;
  let count = 0;
  eventMap[date.toISOString()].photos.forEach((photo) => {
    if (photo.rating) {
      sum += photo.rating;
      count += 1;
    }
  });
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
            label="Jump to"
            v-model="jumpDate"
            @update:model-value="
              () => {
                date[0] = jumpDate;
                setCalendarViewDate(date[0]);
              }
            "
          ></v-date-input>
          <v-calendar
            type="month"
            :events="events"
            hide-week-number
            v-model="date"
            @update:model-value="
              () => {
                console.log(date[0]);
                setCalendarViewDate(date[0]);
              }
            "
          >
            <template v-slot:event="{ day, event }">
              <div class="calendar-photos">
                <photo-icon
                  v-for="photo in event.photos.slice(0, 4)"
                  :key="photo.Id"
                  :photo="photo"
                  :size="100"
                  hide-icons
                  @select="
                    () => {
                      dialogDate = day.date;
                      dayDialog = true;
                    }
                  "
                ></photo-icon>
              </div>
              <div
                v-if="event.photos.length === 0"
                class="focus"
                @click="
                  () => {
                    dialogDate = day.date;
                    dayDialog = true;
                  }
                "
              ></div>
            </template>
          </v-calendar>
        </v-col>
      </v-row>
    </v-container>
    <v-dialog v-model="dayDialog">
      <v-card>
        <v-card-title>{{ dialogDate.toDateString() }}</v-card-title>
        <v-card-text>
          <v-container>
            <v-row>
              <v-col cols="6">
                <photo-icon
                  v-for="photo in eventMap[dialogDate.toISOString()].photos.slice(0, 20)"
                  :key="photo.Id"
                  :photo="photo"
                  :size="100"
                  hide-icons
                  @select="
                    () => {
                      router.push(`/tagger?date=${dialogDate.toISOString()}`);
                    }
                  "
                ></photo-icon>
              </v-col>
              <v-col cols="6">
                <v-chip
                  v-for="place in getLocationsByDate(dialogDate)"
                  :key="place.Id"
                  :color="layers[place.data.layer].data.color"
                  >{{ place.data.name }}</v-chip
                >
                <br />
                Total photos: {{ eventMap[dialogDate.toISOString()].photos.length }}
                <br />
                Average rating: {{ getAvgRatingByDate(dialogDate) }}
              </v-col>
            </v-row>
          </v-container>
        </v-card-text>
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
</style>
