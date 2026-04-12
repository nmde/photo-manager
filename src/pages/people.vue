<script setup lang="ts">
  import type { Person, PersonRec } from '@/classes/Person';
  import { v4 as uuid } from 'uuid';
  import {
    create_person,
    create_person_category,
    get_people,
    get_people_categories,
  } from '@/api/people';
  import { PersonCategory, type PersonCategoryRec } from '@/classes/PersonCategory';

  const router = useRouter();

  const editing = ref(false);
  const editTarget = ref<Person>();
  const addDialog = ref(false);
  const addName = ref('');
  const addCategory = ref('');
  const addCategoryDialog = ref(false);
  const addCategoryName = ref('');
  const addCategoryColor = ref('');
  const saving = ref(false);
  const localCategories = ref<PersonCategoryRec>({});
  const localPeople = ref<PersonRec>({});

  const personCardWidth = 64;
  const personCardHeight = 212;
  const peopleRows = computed(() => {
    const re: Record<string, Person[][]> = {};
    const sortedPeople = Object.values(localPeople.value).toSorted((a, b) => b.count - a.count);
    for (const c in localCategories.value) {
      re[c] = [[]];
      let x = 0;
      for (const person of sortedPeople.filter(p => p.category === c)) {
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

  async function savePerson() {
    saving.value = true;
    if (editing && editTarget.value) {
      if (addName.value !== editTarget.value.name) {
        await editTarget.value.setName(addName.value);
      }
      if (addCategory.value !== editTarget.value.category) {
        await editTarget.value.setCategory(addCategory.value);
      }
    } else {
      await create_person(uuid(), addName.value, addCategory.value);
    }
    saving.value = false;
    addDialog.value = false;
    addName.value = '';
    addCategory.value = '';
  }

  onMounted(async () => {
    await get_people_categories()
      .ok(c => (localCategories.value = c))
      .err(reportError)
      .send();
    await get_people()
      .ok(p => (localPeople.value = p))
      .err(reportError)
      .send();
  });
</script>

<template>
  <v-toolbar color="primary">
    <v-btn
      @click="
        () => {
          editing = false;
          addDialog = true;
        }
      "
    >
      Add Person
    </v-btn>
  </v-toolbar>
  <v-expansion-panels>
    <v-expansion-panel v-for="category in localCategories" :key="category.id">
      <v-expansion-panel-title :color="category.color">{{ category.name }}</v-expansion-panel-title>
      <v-expansion-panel-text>
        <v-virtual-scroll
          :height="640"
          :item-height="personCardHeight"
          :items="peopleRows[category.id]"
        >
          <template #default="{ item }">
            <div class="people-grid">
              <v-card v-for="person in item" :key="person.id">
                <template v-if="person.photo !== null" #prepend>
                  <v-avatar size="128">
                    <v-img :src="person.photo" />
                  </v-avatar>
                </template>
                <v-card-title>
                  {{ person.name }}
                  <v-menu>
                    <template #activator="{ props }">
                      <v-btn flat icon v-bind="props">
                        <v-icon>mdi-pencil</v-icon>
                      </v-btn>
                    </template>
                    <v-list>
                      <v-list-item
                        @click="
                          () => {
                            editing = true;
                            editTarget = person;
                            addName = person.name;
                            addCategory = person.category;
                            addDialog = true;
                          }
                        "
                      >
                        Edit
                      </v-list-item>
                      <v-list-item @click="router.push(`/tagger?person=${person.id}`)">
                        View Photos
                      </v-list-item>
                      <v-list-item @click="router.push(`/tagger?photographer=${person.id}`)">
                        View Photos Taken By
                      </v-list-item>
                    </v-list>
                  </v-menu>
                </v-card-title>
                <v-card-text>
                  Photo count: {{ person.count }}
                  <br />
                  Photos taken: {{ person.photographer_count }}
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
        <color-options @select="color => (addCategoryColor = color ?? '')" />
      </v-card-text>
      <v-card-actions>
        <v-btn @click="addCategoryDialog = false">Cancel</v-btn>
        <v-btn
          color="primary"
          @click="
            async () => {
              const id = uuid();
              await create_person_category(id, addCategoryName, addCategoryColor);
              localCategories[id] = new PersonCategory(id, addCategoryName, addCategoryColor);
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
    <v-card :title="`${editing ? 'Edit' : 'Add'} Person`">
      <v-card-text>
        <v-text-field v-model="addName" color="primary" label="Name" />
        <v-select
          v-model="addCategory"
          item-title="name"
          item-value="id"
          :items="Object.values(localCategories)"
          label="Category"
        >
          <template #item="{ props, item }">
            <v-list-item v-bind="props" :base-color="item.color" />
          </template>
        </v-select>
      </v-card-text>
      <v-card-actions>
        <v-btn @click="addDialog = false">Cancel</v-btn>
        <v-btn color="primary" :loading="saving" @click="async () => savePerson()">Save</v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
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
