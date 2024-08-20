<script lang="ts" setup>
import type { Activity } from '~/classes/Activity';
import { fileStore, moods } from '../../stores/fileStore';

const route = useRoute();

const {
  createJournalEntry,
  journals,
  activities,
  createActivity,
  setCalendarViewDate,
  calendarViewDate,
} = fileStore;

function simplifyDate(date: Date) {
  return new Date(date.getFullYear(), date.getMonth(), date.getDate());
}

const createDialog = ref(false);
const createDate = ref(simplifyDate(new Date()));
const createMood = ref(2);
const createText = ref('');
const createActivities = ref<number[]>([]);
const createSteps = ref('0');

const date = ref(simplifyDate(new Date()));
const mood = ref(2);
const text = ref('');
const entryActivities = ref<Activity[]>([]);
const steps = ref(0);

function setDate(d: Date) {
  date.value = simplifyDate(d);
  if (journals[date.value.toISOString()]) {
    const entry = journals[date.value.toISOString()];
    mood.value = entry.data.mood;
    text.value = entry.data.text;
    entryActivities.value = entry.activities;
    steps.value = entry.data.steps;
  } else {
    mood.value = 2;
    text.value = '';
    entryActivities.value = [];
    steps.value = 0;
  }
  setCalendarViewDate(date.value);
}

const activityDialog = ref(false);
const activityName = ref('');
const activityIcon = ref('');
const localActivities = ref<Activity[]>([]);

onMounted(() => {
  if (typeof route.query.date === 'string') {
    date.value = simplifyDate(new Date(route.query.date));
  } else {
    date.value = calendarViewDate;
  }
  setDate(date.value);
  localActivities.value = Object.values(activities);
});
</script>

<template>
  <v-main class="main">
    <v-btn
      icon
      flat
      @click="setDate(new Date(date.getFullYear(), date.getMonth(), date.getDate() - 1))"
    >
      <v-icon>mdi-arrow-left</v-icon>
    </v-btn>
    <v-btn
      icon
      flat
      @click="setDate(new Date(date.getFullYear(), date.getMonth(), date.getDate() + 1))"
    >
      <v-icon>mdi-arrow-right</v-icon>
    </v-btn>
    <h1 class="header">
      {{ date.toISOString() }}
      <mood-icon :mood="mood"></mood-icon>
      <v-btn
        icon
        @click="
          () => {
            createDate = date;
            createMood = mood;
            createText = text;
            const a: number[] = [];
            Object.values(entryActivities).forEach((activity) => {
              a.push(Object.keys(activities).indexOf(activity.Id));
            });
            createActivities = a;
            createSteps = `${steps}`;
            createDialog = true;
          }
        "
      >
        <v-icon>mdi-pencil</v-icon>
      </v-btn>
    </h1>
    <v-chip
      v-for="activity in entryActivities"
      :key="activity.Id"
      :text="activity.data.name"
      :prepend-icon="activity.data.icon"
    ></v-chip>
    {{ steps }} Steps<br />
    <p class="entry-text">{{ text }}</p>
  </v-main>
  <v-dialog v-model="createDialog">
    <v-card>
      <v-card-title>Add an Entry</v-card-title>
      <v-card-text>
        <v-date-input label="Date" v-model="createDate"></v-date-input>
        <v-select
          label="Mood"
          :items="moods"
          item-title="label"
          item-value="value"
          v-model="createMood"
        >
          <template v-slot:item="{ item, props }">
            <v-list-item v-bind="props" :style="{ color: item.raw.color }"></v-list-item>
          </template>
        </v-select>
        <v-chip-group multiple column selected-class="text-primary" v-model="createActivities">
          <v-chip
            v-for="activity in localActivities"
            :key="activity.Id"
            :text="activity.data.name"
            :prepend-icon="activity.data.icon"
          ></v-chip>
          <v-btn flat icon @click="activityDialog = true">
            <v-icon>mdi-plus</v-icon>
          </v-btn>
        </v-chip-group>
        <v-text-field v-model="createSteps" label="Steps"></v-text-field>
        <v-textarea v-model="createText"></v-textarea>
      </v-card-text>
      <v-card-actions>
        <v-btn @click="createDialog = false">Cancel</v-btn>
        <v-btn
          @click="
            async () => {
              const j = await createJournalEntry(
                createDate.toISOString(),
                createMood,
                createText,
                createActivities.map((i) => Object.values(activities)[i]),
                Number(createSteps),
              );
              mood = createMood;
              text = createText;
              date = createDate;
              steps = Number(createSteps);
              entryActivities = j.activities;
              createDialog = false;
              createMood = 2;
              createText = '';
              createActivities = [];
              createSteps = '0';
            }
          "
          color="primary"
          >Save</v-btn
        >
      </v-card-actions>
    </v-card>
  </v-dialog>
  <v-dialog v-model="activityDialog">
    <v-card>
      <v-card-title>Add Activity</v-card-title>
      <v-card-text>
        <v-text-field v-model="activityName" label="Name"></v-text-field>
        <v-text-field v-model="activityIcon" label="Icon"></v-text-field>
      </v-card-text>
      <v-card-actions>
        <v-btn @click="activityDialog = false">Cancel</v-btn>
        <v-btn
          @click="
            async () => {
              const a = await createActivity(activityName, activityIcon);
              activityDialog = false;
              activityName = '';
              activityIcon = '';
              localActivities.push(a);
            }
          "
          color="primary"
          >Save</v-btn
        >
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<style scoped>
.main {
  margin: 8px;
}

.header {
  display: flex;
}

.entry-text {
  white-space: pre-wrap;
}
</style>
