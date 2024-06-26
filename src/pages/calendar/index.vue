<script setup lang="ts">
import { computed } from 'vue';
import { fileStore } from '../../stores/fileStore';
import { Photo } from '../../classes/Photo';

const router = useRouter();

const jumpDate = ref<Date>(new Date());
const date = ref<Date[]>([new Date()]);

const { files, calendarViewDate, setCalendarViewDate } = fileStore;

const events = computed(() => {
  const events: any[] = [];
  const eventMap: Record<
    string,
    {
      date: Date;
      photos: Photo[];
    }
  > = {};
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
        if (eventMap[k].photos.length > 4) {
          eventMap[k].photos.pop();
        }
        if (photo.group !== undefined) {
          groups.push(photo.group);
        }
      }
    }
  });
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
                setCalendarViewDate(date[0]);
              }
            "
          >
            <template v-slot:event="{ day, event }">
              <div class="calendar-photos">
                <photo-icon
                  v-for="photo in event.photos"
                  :key="photo.Id"
                  :photo="photo"
                  :size="100"
                  hide-icons
                  @select="
                    () => {
                      router.push(`/tagger?date=${day.isoDate}`);
                    }
                  "
                ></photo-icon>
              </div>
            </template>
          </v-calendar>
        </v-col>
      </v-row>
    </v-container>
  </v-main>
</template>
