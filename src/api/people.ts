import { invoke } from '@tauri-apps/api/core';
import { APIResult } from '@/classes/APIResult';
import { Person, type PersonData, type PersonRec } from '@/classes/Person';
import {
  PersonCategory,
  type PersonCategoryData,
  type PersonCategoryRec,
} from '@/classes/PersonCategory';

export async function create_person(
  id: PersonData['id'],
  name: PersonData['name'],
  category: PersonData['category'],
) {
  await invoke('create_person', { id, name, category });
}

export async function create_person_category(
  id: PersonCategoryData['id'],
  name: PersonCategoryData['name'],
  color: PersonCategoryData['color'],
) {
  await invoke('create_person_category', { id, name, color });
}

export async function set_person_name(person: PersonData['id'], value: PersonData['name']) {
  await invoke('set_person_name', { person, value });
}

export async function set_person_category(person: PersonData['id'], value: PersonData['category']) {
  await invoke('set_person_category', { person, value });
}

export async function set_person_photo(person: PersonData['id'], value: PersonData['photo']) {
  await invoke('set_person_photo', { person, value });
}

export function get_people() {
  return new APIResult<PersonData[], PersonRec>(
    async () => await invoke('get_people'),
    people => Person.createPeople(people),
  );
}

export function get_people_categories() {
  return new APIResult<PersonCategoryData[], PersonCategoryRec>(
    async () => await invoke('get_people_categories'),
    categories => PersonCategory.createCategories(categories),
  );
}
