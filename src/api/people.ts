import { invoke } from '@tauri-apps/api/core';
import { APIResult } from '@/classes/APIResult';
import { Person, type PersonData } from '@/classes/Person';
import { PersonCategory } from '@/classes/PersonCategory';

export async function create_person(id: string, name: string, category: string) {
  await invoke('create_person', { id, name, category });
}

export async function create_person_category(id: string, name: string, color: string) {
  await invoke('create_person_category', { id, name, color });
}

export async function set_person_name(person: string, value: string) {
  await invoke('set_person_name', { person, value });
}

export async function set_person_category(person: string, value: string) {
  await invoke('set_person_category', { person, value });
}

export async function set_person_photo(person: string, value: string | null) {
  await invoke('set_person_photo', { person, value });
}

export function get_people() {
  return new APIResult<PersonData[], Record<string, Person>>(
    async () => await invoke('get_people'),
    people => Person.createPeople(people),
  );
}

export function get_people_categories() {
  return new APIResult<PersonCategory[], Record<string, PersonCategory>>(
    async () => await invoke('get_people_categories'),
    categories => PersonCategory.createCategories(categories),
  );
}
