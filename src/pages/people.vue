<script setup lang="ts">
  import type { Person, PersonRec } from '@/classes/Person';
  import { v4 as uuid } from 'uuid';
  import { useRules } from 'vuetify/labs/rules';
  import {
    create_person,
    create_person_category,
    get_people,
    get_people_categories,
  } from '@/api/people';
  import { PersonCategory, type PersonCategoryRec } from '@/classes/PersonCategory';

  const router = useRouter();
  const rules = useRules();

  const editing = ref(false);
  const editTarget = ref<Person>();
  const addDialog = ref(false);
  const addCategoryDialog = ref(false);
  const localCategories = ref<PersonCategoryRec>({});
  const localPeople = ref<PersonRec>({});
  const missingCategoryColor = ref(false);

  type AddPersonFields = {
    name?: string;
    category?: string;
  };
  const addPersonFields = ref<AddPersonFields>({});

  type AddCategoryFields = {
    name?: string;
    color?: string;
  };
  const addCategoryFields = ref<AddCategoryFields>({});

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
    const fields = addPersonFields.value as Required<AddPersonFields>;
    if (editing && editTarget.value) {
      if (fields.name !== editTarget.value.name) {
        await editTarget.value.setName(fields.name);
      }
      if (fields.category !== editTarget.value.category) {
        await editTarget.value.setCategory(fields.category);
      }
    } else {
      await create_person(uuid(), fields.name, fields.category);
    }
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
                            addPersonFields = person;
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
  <form-dialog
    v-model="addCategoryDialog"
    :reset="() => (addCategoryFields = {})"
    title="Add Category"
    @submit="
      async () => {
        missingCategoryColor = addCategoryFields.color === undefined;
        if (!missingCategoryColor) {
          const id = uuid();
          const fields = addCategoryFields as Required<AddCategoryFields>;
          await create_person_category(id, fields.name, fields.color);
          localCategories[id] = new PersonCategory(id, fields.name, fields.color);
        }
      }
    "
  >
    <v-text-field
      v-model="addCategoryFields.name"
      label="Name"
      :rules="[rules.required('A name is required.')]"
    />
    <color-options
      :error="missingCategoryColor"
      :value="addCategoryFields.color"
      @select="color => (addCategoryFields.color = color ?? undefined)"
    />
  </form-dialog>
  <form-dialog
    v-model="addDialog"
    :reset="() => (addPersonFields = {})"
    :title="`${editing ? 'Edit' : 'Add'} Person`"
    @submit="async () => savePerson()"
  >
    <v-text-field v-model="addPersonFields.name" color="primary" label="Name" />
    <v-select
      v-model="addPersonFields.category"
      item-title="name"
      item-value="id"
      :items="Object.values(localCategories)"
      label="Category"
      :rules="[rules.required('A category is required.')]"
    >
      <template #item="{ props, item }">
        <v-list-item v-bind="props" :base-color="item.color" />
      </template>
    </v-select>
  </form-dialog>
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
