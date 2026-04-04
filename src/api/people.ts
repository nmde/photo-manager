import { Person, type PersonData } from '@/classes/Person';
import { PersonCategory } from '@/classes/PersonCategory';
import { invoke } from '@tauri-apps/api/core';

export const create_person = async (id: string, name: string, category: string) =>
  await invoke('create_person', { id, name, category });

export const create_person_category = async (id: string, name: string, color: string) =>
  await invoke('create_person_category', { id, name, color });

export const set_person_name = async (person: string, value: string) =>
  await invoke('set_person_name', { person, value });

export const set_person_category = async (person: string, value: string) =>
  await invoke('set_person_category', { person, value });

export const set_person_notes = async (person: string, value: string) =>
  await invoke('set_person_notes', { person, value });

export const set_person_photo = async (person: string, value: string) =>
  await invoke('set_person_photo', { person, value });

export const get_people = async () =>
  Person.createPeople(await invoke<Record<string, PersonData>>('get_people'));

export const get_people_categories = async () =>
  PersonCategory.createCategories(await invoke<PersonCategory[]>('get_people_categories'));
