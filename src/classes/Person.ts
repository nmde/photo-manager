import { invoke } from '@tauri-apps/api/core';

export class Person {
  public count = 0;

  public photographerCount = 0;

  public constructor(
    private _id: string,
    private _name: string,
    private _photo: string,
    private _notes: string,
    private _category: string,
  ) {}

  public get id() {
    return this._id;
  }

  public get name() {
    return this._name;
  }

  public get photo() {
    return this._photo;
  }

  public get notes() {
    return this._notes;
  }

  public get category() {
    return this._category;
  }

  public async setName(name: string) {
    this._name = name;
    await invoke('set_person_str', {
      person: this._id,
      property: 'name',
      value: name,
    });
  }

  public async setNotes(notes: string) {
    this._notes = notes;
    await invoke('set_person_str', {
      person: this._id,
      property: 'notes',
      value: notes,
    });
  }

  public async setCategory(category: string) {
    this._category = category;
    await invoke('set_person_str', {
      person: this._id,
      property: 'category',
      value: category,
    });
  }

  public async setPhoto(photo: string) {
    this._photo = photo;
    await invoke('set_person_str', {
      person: this._id,
      property: 'photo',
      value: photo,
    });
  }
}
