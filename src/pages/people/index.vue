<script setup lang="ts">
import { onMounted, ref, computed } from 'vue';
import { Person } from '../../classes/Person';
import { PersonCategory } from '../../classes/PersonCategory';
import { fileStore } from '../../stores/fileStore';

const { addPerson, peopleMap, peopleCategories, addPersonCategory } = fileStore;

const addDialog = ref(false);
const addName = ref('');
const addNotes = ref('');
const addCategory = ref('');

const addCategoryDialog = ref(false);
const addCategoryName = ref('');
const addCategoryColor = ref('');

const localCategories = ref<Record<string, PersonCategory>>({});
const localPeople = ref<Record<string, Person[]>>({});
const categoryList = computed(() =>
  Object.values(localCategories.value).map((c) => ({
    color: c.data.color,
    title: c.data.name,
    value: c.Id,
  })),
);

onMounted(() => {
  localCategories.value = peopleCategories;
  localPeople.value = peopleMap;
});
</script>

<template>
  <v-main>
    <v-btn color="primary" @click="addDialog = true">Add Person</v-btn>
    <v-btn color="secondary" @click="addCategoryDialog = true">Add Category</v-btn>
    <v-expansion-panels>
      <v-expansion-panel v-for="category in localCategories" :key="category.Id">
        <v-expansion-panel-title>{{ category.data.name }}</v-expansion-panel-title>
        <v-expansion-panel-text>
          <v-card v-for="person in localPeople[category.Id]" :key="person.Id">
            <template v-slot:prepend v-if="person.data.photo.length > 0">
              <v-avatar size="24">
                <v-img :src="person.data.photo"></v-img>
              </v-avatar>
            </template>
            <v-card-title>{{ person.data.name }}</v-card-title>
            <v-card-text>{{ person.data.notes }}</v-card-text>
          </v-card>
        </v-expansion-panel-text>
      </v-expansion-panel>
    </v-expansion-panels>
    <v-dialog v-model="addCategoryDialog">
      <v-card>
        <v-card-title>Add Category</v-card-title>
        <v-card-text>
          <v-text-field label="Name" v-model="addCategoryName"></v-text-field>
          <color-options @select="(color: string) => (addCategoryColor = color)"></color-options>
        </v-card-text>
        <v-card-actions>
          <v-btn @click="addCategoryDialog = false">Cancel</v-btn>
          <v-btn
            color="primary"
            @click="
              async () => {
                const c = await addPersonCategory(addCategoryName, addCategoryColor);
                localCategories[c.Id] = c;
                localPeople[c.Id] = [];
                addCategoryDialog = false;
                addCategoryName = '';
                addCategoryColor = '';
              }
            "
            >Save</v-btn
          >
        </v-card-actions>
      </v-card>
    </v-dialog>
    <v-dialog v-model="addDialog">
      <v-card>
        <v-card-title>Add Person</v-card-title>
        <v-card-text>
          <v-text-field v-model="addName" label="Name"></v-text-field>
          <v-select label="Category" :items="categoryList" v-model="addCategory">
            <template v-slot:item="{ props, item }">
              <v-list-item v-bind="props" :base-color="item.raw.color"></v-list-item>
            </template>
          </v-select>
          <v-textarea v-model="addNotes" label="Notes"></v-textarea>
        </v-card-text>
        <v-card-actions>
          <v-btn @click="addDialog = false">Cancel</v-btn>
          <v-btn
            color="primary"
            @click="
              async () => {
                const p = await addPerson(addName, addNotes, addCategory);
                addDialog = false;
                addName = '';
                addNotes = '';
                addCategory = '';
              }
            "
            >Save</v-btn
          >
        </v-card-actions>
      </v-card>
    </v-dialog>
  </v-main>
</template>
