<script lang="ts" setup>
import { ref, onMounted } from 'vue';
import { useRoute } from 'vue-router';
import type { Activity } from '@/classes/Activity';
import { fileStore, formatDate, moods } from '@/stores/fileStore';
import MoodIcon from '@/components/MoodIcon.vue';
import DecryptionDialog from '@/components/DecryptionDialog.vue';
import MarkdownEditor from '@/components/MarkdownEditor.vue';

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

const decryptDialog = ref(false);
const encryptionBlock = ref(false);

fileStore.on('decrypted', () => {
  encryptionBlock.value = false;
  text.value = journals[formatDate(date.value)].data.text;
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
    <div v-if="encryptionBlock">
      <h3>Journal entries are encrypted!</h3>
      <v-btn @click="decryptDialog = true" color="primary">Decrypt Entries</v-btn>
    </div>
    <MarkdownEditor
      v-else
      :text="text"
      @save="
        async (newText) => {
          await createJournalEntry(
            formatDate(date),
            mood,
            newText,
            entryActivities,
            steps,
          );
          createText = newText;
        }
      "
    ></MarkdownEditor>
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
      </v-card-text>
      <v-card-actions>
        <v-btn @click="createDialog = false">Cancel</v-btn>
        <v-btn
          @click="
            async () => {
              // TODO: The way this is set up, when you modify activities after typing in the text area,
              // the text variable isn't synced and replaces the text with whatever was or was not there before you click save
              const j = await createJournalEntry(
                formatDate(createDate),
                createMood,
                createText,
                createActivities.map((i) => Object.values(activities)[i]),
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
  <DecryptionDialog :value="decryptDialog"></DecryptionDialog>
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
