<script lang="ts" setup>
import type { Activity } from '~/classes/Activity';
import { fileStore, moods } from '../../stores/fileStore';

const route = useRoute();

const { createJournalEntry, journals, activities, createActivity } = fileStore;

function simplifyDate(date: Date) {
  return new Date(date.getFullYear(), date.getMonth(), date.getDate());
}

const createDialog = ref(false);
const createDate = ref(simplifyDate(new Date()));
const createMood = ref(2);
const createText = ref('');
const createActivities = ref<number[]>([]);

const date = ref(simplifyDate(new Date()));
const mood = ref(2);
const text = ref('');
const entryActivities = ref<Activity[]>([]);

function setDate(d: Date) {
  date.value = simplifyDate(d);
  if (journals[date.value.toISOString()]) {
    const entry = journals[date.value.toISOString()];
    mood.value = entry.data.mood;
    text.value = entry.data.text;
    entryActivities.value = entry.activities;
  } else {
    mood.value = 2;
    text.value = '';
    entryActivities.value = [];
  }
}

const activityDialog = ref(false);
const activityName = ref('');
const activityIcon = ref('');
const localActivities = ref<Activity[]>([]);

onMounted(() => {
  if (typeof route.query.date === 'string') {
    date.value = simplifyDate(new Date(route.query.date));
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
    <v-btn
      @click="
        () => {
          createDate = date;
          createDialog = true;
        }
      "
      >Add Entry</v-btn
    >
    <h1 class="header">
      {{ date.toISOString() }}
      <mood-icon :mood="mood"></mood-icon>
    </h1>
    <v-chip
      v-for="activity in entryActivities"
      :key="activity.Id"
      :text="activity.data.name"
      :prepend-icon="activity.data.icon"
    ></v-chip>
    {{ text }}
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
        <v-chip-group multiple selected-class="text-primary" v-model="createActivities">
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
        <v-textarea v-model="createText"></v-textarea>
      </v-card-text>
      <v-card-actions>
        <v-btn @click="createDialog = false">Cancel</v-btn>
        <v-btn
          @click="
            async () => {
              await createJournalEntry(
                createDate.toISOString(),
                createMood,
                createText,
                createActivities.map((i) => Object.values(activities)[i]),
              );
              mood = createMood;
              text = createText;
              date = createDate;
              createDialog = false;
              createMood = 2;
              createText = '';
              createActivities = [];
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
</style>
