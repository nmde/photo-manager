<script setup lang="ts">
  import type { Person } from '../classes/Person';
  import type { PersonCategory } from '../classes/PersonCategory';
  import { computed, onMounted, ref } from 'vue';
  import { useRouter } from 'vue-router';
  import { fileStore } from '../stores/fileStore';

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
    Object.values(localCategories.value).map(c => ({
      color: c.color,
      title: c.name,
      value: c.id,
    })),
  );

  const personCardWidth = 64;
  const personCardHeight = 212;
  const peopleRows = computed(() => {
    const re: Record<string, Person[][]> = {};
    for (const [c, category] of Object.entries(localPeople.value)) {
      re[c] = [[]];
      let x = 0;
      for (const person of category) {
        re[c].at(-1)?.push(person);
        x += 1;
        if (x > window.innerWidth / personCardWidth) {
          x = 0;
          re[c].push([]);
        }
      }
    }
    return re;
  });

  onMounted(() => {
    localCategories.value = peopleCategories;
    localPeople.value = {};
    for (const [category, people] of Object.entries(peopleMap)) {
      localPeople.value[category] = people.toSorted((a, b) => b.count - a.count);
    }
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
    >
      Add Person
    </v-btn>
    <v-btn color="secondary" @click="addCategoryDialog = true">Add Category</v-btn>
    <v-expansion-panels>
      <v-expansion-panel v-for="category in localCategories" :key="category.id">
        <v-expansion-panel-title :color="category.color">{{
          category.name
        }}</v-expansion-panel-title>
        <v-expansion-panel-text>
          <v-virtual-scroll
            :height="640"
            :item-height="personCardHeight"
            :items="peopleRows[category.id]"
          >
            <template #default="{ item }">
              <div class="people-grid">
                <v-card v-for="person in item" :key="person.id" class="person-card">
                  <template v-if="person.photo.length > 0" #prepend>
                    <v-avatar size="128">
                      <v-img :src="person.photo" />
                    </v-avatar>
                  </template>
                  <v-card-title>
                    {{ person.name }}
                    <v-menu>
                      <template #activator="{ props }">
                        <v-btn flat icon v-bind="props">
                          <v-icon>mdi-menu</v-icon>
                        </v-btn>
                      </template>
                      <v-list>
                        <v-list-item
                          @click="
                            () => {
                              editing = true;
                              editTarget = person.id;
                              addName = person.name;
                              addNotes = person.notes;
                              addCategory = person.category;
                              addDialog = true;
                            }
                          "
                        >
                          Edit
                        </v-list-item>
                        <v-list-item
                          @click="
                            () => {
                              router.push(`/tagger?person=${person.id}`);
                            }
                          "
                        >
                          View Photos
                        </v-list-item>
                        <v-list-item
                          @click="
                            () => {
                              router.push(`/tagger?photographer=${person.id}`);
                            }
                          "
                        >
                          View Photos Taken By
                        </v-list-item>
                      </v-list>
                    </v-menu>
                  </v-card-title>
                  <v-card-text>
                    Photo count: {{ person.count }}
                    <br>
                    Photos taken: {{ person.photographerCount }}
                    <br>
                    <p class="notes">{{ person.notes }}</p>
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
          <v-text-field v-model="addCategoryName" label="Name" />
          <color-options @select="color => (addCategoryColor = color)" />
        </v-card-text>
        <v-card-actions>
          <v-btn @click="addCategoryDialog = false">Cancel</v-btn>
          <v-btn
            color="primary"
            @click="
              async () => {
                const c = await addPersonCategory(addCategoryName, addCategoryColor);
                localCategories[c.id] = c;
                localPeople[c.id] = [];
                addCategoryDialog = false;
                addCategoryName = '';
                addCategoryColor = '';
              }
            "
          >
            Save
          </v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>
    <v-dialog v-model="addDialog">
      <v-card>
        <v-card-title>Add Person</v-card-title>
        <v-card-text>
          <v-text-field v-model="addName" label="Name" />
          <v-select v-model="addCategory" :items="categoryList" label="Category">
            <template #item="{ props, item }">
              <v-list-item v-bind="props" :base-color="item.raw.color" />
            </template>
          </v-select>
          <v-textarea v-model="addNotes" label="Notes" />
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
          >
            Save
          </v-btn>
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
