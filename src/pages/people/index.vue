<script setup lang="ts">
import { onMounted, ref, computed } from 'vue';
import { Person } from '../../classes/Person';
import { PersonCategory } from '../../classes/PersonCategory';
import { fileStore } from '../../stores/fileStore';

const router = useRouter();

const { addPerson, peopleMap, peopleCategories, addPersonCategory, updatePerson } = fileStore;

const editing = ref(false);
const editTarget = ref('');
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

const personCardWidth = 64;
const personCardHeight = 212;
const peopleRows = computed(() => {
  const re: Record<string, Person[][]> = {};
  Object.entries(localPeople.value).forEach(([c, category]) => {
    re[c] = [[]];
    let x = 0;
    category.forEach((person) => {
      re[c][re[c].length - 1].push(person);
      x += 1;
      if (x > window.innerWidth / personCardWidth) {
        x = 0;
        re[c].push([]);
      }
    });
  });
  return re;
});

onMounted(() => {
  localCategories.value = peopleCategories;
  localPeople.value = {};
  Object.entries(peopleMap).forEach(([category, people]) => {
    localPeople.value[category] = people.sort((a, b) => {
      if (a.count < b.count) {
        return 1;
      }
      if (a.count > b.count) {
        return -1;
      }
      return 0;
    });
  });
});
</script>

<template>
  <v-main>
    <v-btn
      color="primary"
      @click="
        () => {
          editing = false;
          addDialog = true;
        }
      "
      >Add Person</v-btn
    >
    <v-btn color="secondary" @click="addCategoryDialog = true">Add Category</v-btn>
    <v-expansion-panels>
      <v-expansion-panel v-for="category in localCategories" :key="category.Id">
        <v-expansion-panel-title :color="category.data.color">{{
          category.data.name
        }}</v-expansion-panel-title>
        <v-expansion-panel-text>
          <v-virtual-scroll
            :height="640"
            :item-height="personCardHeight"
            :items="peopleRows[category.Id]"
          >
            <template v-slot:default="{ item }">
              <div class="people-grid">
                <v-card class="person-card" v-for="person in item" :key="person.Id">
                  <template v-slot:prepend v-if="person.data.photo.length > 0">
                    <v-avatar size="128">
                      <v-img :src="person.data.photo"></v-img>
                    </v-avatar>
                  </template>
                  <v-card-title
                    >{{ person.data.name }}
                    <v-menu>
                      <template v-slot:activator="{ props }">
                        <v-btn icon flat v-bind="props">
                          <v-icon>mdi-menu</v-icon>
                        </v-btn>
                      </template>
                      <v-list>
                        <v-list-item
                          @click="
                            () => {
                              editing = true;
                              editTarget = person.Id;
                              addName = person.data.name;
                              addNotes = person.data.notes;
                              addCategory = person.data.category;
                              addDialog = true;
                            }
                          "
                          >Edit</v-list-item
                        >
                        <v-list-item
                          @click="
                            () => {
                              router.push(`/tagger?person=${person.Id}`);
                            }
                          "
                        >
                          View Photos
                        </v-list-item>
                        <v-list-item
                          @click="
                            () => {
                              router.push(`/tagger?photographer=${person.Id}`);
                            }
                          "
                          >View Photos Taken By</v-list-item
                        >
                      </v-list>
                    </v-menu>
                  </v-card-title>
                  <v-card-text>
                    Photo count: {{ person.count }}
                    <br />
                    Photos taken: {{ person.photographerCount }}
                    <br />
                    <p class="notes">{{ person.data.notes }}</p>
                  </v-card-text>
                </v-card>
              </div>
            </template>
          </v-virtual-scroll>
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
                if (editing) {
                  await updatePerson(editTarget, addName, addNotes, addCategory);
                } else {
                  await addPerson(addName, addNotes, addCategory);
                }
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

<style scoped>
.people-grid {
  display: flex;
  flex-wrap: wrap;
}

.person-card {
  margin: 12px;
}

.notes {
  white-space: pre;
}
</style>
