import { Person, type PersonData } from '@/classes/Person';
import { PersonCategory } from '@/classes/PersonCategory';
import { invoke } from '@tauri-apps/api/core';

export const get_people = async () =>
  Person.createPeople(await invoke<Record<string, PersonData>>('get_people'));

export const get_people_categories = async () =>
  PersonCategory.createCategories(await invoke<PersonCategory[]>('get_people_categories'));
