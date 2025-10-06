<script lang="ts" setup>
import type { Activity } from '@/classes/Activity';
import { onMounted, ref } from 'vue';
import { useRoute } from 'vue-router';
import { fileStore, formatDate, moods } from '@/stores/fileStore';

const route = useRoute('/journal/[date]');

const {
  createJournalEntry,
  journals,
  activities,
  createActivity,
  setCalendarViewDate,
  calendarViewDate,
  encrypted,
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
  if (journals[formatDate(date.value)]) {
    const entry = journals[formatDate(date.value)];
    if (entry) {
      mood.value = entry.data.mood;
      text.value = entry.data.text;
      entryActivities.value = entry.activities;
      steps.value = entry.data.steps;
    }
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

const decryptDialog = ref(false);
const encryptionBlock = ref(false);

fileStore.on('decrypted', () => {
  encryptionBlock.value = false;
  text.value = journals[formatDate(date.value)]?.data.text ?? '';
});

onMounted(() => {
  if (typeof route.params.date === 'string') {
    const split = route.params.date.split('-');
    date.value = simplifyDate(new Date(Number(split[0]), Number(split[1]) - 1, Number(split[2])));
  } else {
    date.value = calendarViewDate;
  }
  setDate(date.value);
  localActivities.value = Object.values(activities);
  if (encrypted) {
    encryptionBlock.value = true;
  }
});
</script>

<template>
  <v-main class="main">
    <v-btn
      flat
      icon
      @click="setDate(new Date(date.getFullYear(), date.getMonth(), date.getDate() - 1))"
    >
      <v-icon>mdi-arrow-left</v-icon>
    </v-btn>
    <v-btn
      flat
      icon
      @click="setDate(new Date(date.getFullYear(), date.getMonth(), date.getDate() + 1))"
    >
      <v-icon>mdi-arrow-right</v-icon>
    </v-btn>
    <h1 class="header">
      {{ date.toISOString() }}
      <mood-icon :mood="mood" />
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
      :prepend-icon="activity.data.icon"
      :text="activity.data.name"
    />
    {{ steps }} Steps<br />
    <div v-if="encryptionBlock">
      <h3>Journal entries are encrypted!</h3>
      <v-btn color="primary" @click="decryptDialog = true">Decrypt Entries</v-btn>
    </div>
    <MarkdownEditor
      v-else
      :text="text"
      @save="
        async newText => {
          await createJournalEntry(formatDate(date), mood, newText, entryActivities, steps);
          createText = newText;
        }
      "
    />
  </v-main>
  <v-dialog v-model="createDialog">
    <v-card>
      <v-card-title>Add an Entry</v-card-title>
      <v-card-text>
        <v-date-input v-model="createDate" label="Date" />
        <v-select
          v-model="createMood"
          item-title="label"
          item-value="value"
          :items="moods"
          label="Mood"
        >
          <template #item="{ item, props: lprops }">
            <v-list-item v-bind="lprops" :style="{ color: item.raw.color }" />
          </template>
        </v-select>
        <v-chip-group v-model="createActivities" column multiple selected-class="text-primary">
          <v-chip
            v-for="activity in localActivities"
            :key="activity.Id"
            :prepend-icon="activity.data.icon"
            :text="activity.data.name"
          />
          <v-btn flat icon @click="activityDialog = true">
            <v-icon>mdi-plus</v-icon>
          </v-btn>
        </v-chip-group>
        <v-text-field v-model="createSteps" label="Steps" />
      </v-card-text>
      <v-card-actions>
        <v-btn @click="createDialog = false">Cancel</v-btn>
        <v-btn
          color="primary"
          @click="
            async () => {
              // TODO: The way this is set up, when you modify activities after typing in the text area,
              // the text variable isn't synced and replaces the text with whatever was or was not there before you click save
              const j = await createJournalEntry(
                formatDate(createDate),
                createMood,
                createText,
                createActivities
                  .map(i => Object.values(activities)[i])
                  .filter(i => i !== undefined),
                Number(createSteps),
              );
              mood = createMood;
              date = createDate;
              steps = Number(createSteps);
              entryActivities = j.activities;
              createDialog = false;
              createMood = 2;
              createActivities = [];
              createSteps = '0';
            }
          "
        >
          Save
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
  <v-dialog v-model="activityDialog">
    <v-card>
      <v-card-title>Add Activity</v-card-title>
      <v-card-text>
        <v-text-field v-model="activityName" label="Name" />
        <v-text-field v-model="activityIcon" label="Icon" />
      </v-card-text>
      <v-card-actions>
        <v-btn @click="activityDialog = false">Cancel</v-btn>
        <v-btn
          color="primary"
          @click="
            async () => {
              const a = await createActivity(activityName, activityIcon);
              activityDialog = false;
              activityName = '';
              activityIcon = '';
              localActivities.push(a);
            }
          "
        >
          Save
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
  <DecryptionDialog :value="decryptDialog" />
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
