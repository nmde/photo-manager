<script lang="ts" setup>
import { fileStore, moods } from '../../stores/fileStore';

const route = useRoute();

const { createJournalEntry, journals } = fileStore;

function simplifyDate(date: Date) {
  return new Date(date.getFullYear(), date.getMonth(), date.getDate());
}

const createDialog = ref(false);
const createDate = ref(simplifyDate(new Date()));
const createMood = ref(2);
const createText = ref('');

const date = ref(simplifyDate(new Date()));
const mood = ref(2);
const text = ref('');

function setDate(d: Date) {
  date.value = simplifyDate(d);
  if (journals[date.value.toISOString()]) {
    const entry = journals[date.value.toISOString()];
    mood.value = entry.data.mood;
    text.value = entry.data.text;
  }
}

onMounted(() => {
  if (typeof route.query.date === 'string') {
    date.value = simplifyDate(new Date(route.query.date));
  }
  setDate(date.value);
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
    <v-btn @click="createDialog = true">Add Entry</v-btn>
    <h1 class="header">
      {{ date.toISOString() }}
      <mood-icon :mood="mood"></mood-icon>
    </h1>
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
        <v-textarea v-model="createText"></v-textarea>
      </v-card-text>
      <v-card-actions>
        <v-btn @click="createDialog = false">Cancel</v-btn>
        <v-btn
          @click="
            async () => {
              await createJournalEntry(createDate.toISOString(), createMood, createText);
              mood = createMood;
              text = createText;
              date = createDate;
              createDialog = false;
              createMood = 2;
              createText = '';
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
