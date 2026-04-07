import { invoke } from '@tauri-apps/api/core';
import { Person, type PersonData } from '@/classes/Person';
import { PersonCategory } from '@/classes/PersonCategory';

export async function create_person(id: string, name: string, category: string) {
  return await invoke('create_person', { id, name, category });
}

export async function create_person_category(id: string, name: string, color: string) {
  return await invoke('create_person_category', { id, name, color });
}

export async function set_person_name(person: string, value: string) {
  return await invoke('set_person_name', { person, value });
}

export async function set_person_category(person: string, value: string) {
  return await invoke('set_person_category', { person, value });
}

export async function set_person_photo(person: string, value: string) {
  return await invoke('set_person_photo', { person, value });
}

export async function get_people() {
  return Person.createPeople(await invoke<PersonData[]>('get_people'));
}

export async function get_people_categories() {
  return PersonCategory.createCategories(await invoke<PersonCategory[]>('get_people_categories'));
}
