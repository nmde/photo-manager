import {
  set_person_category,
  set_person_name,
  set_person_notes,
  set_person_photo,
} from '@/api/people';

export type PersonData = {
  id: string;
  name: string;
  photo: string;
  notes: string;
  category: string;
  photographer_count: number;
  photo_count: number;
};

export class Person {
  public constructor(
    public readonly id: string,
    private _name: string,
    private _photo: string,
    private _notes: string,
    private _category: string,
    public photographerCount: number,
    public count: number,
  ) {}

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

  public static createPeople(people: Record<string, PersonData>) {
    const mapped: Record<string, Person> = {};
    for (const person in people) {
      mapped[person] = new Person(
        person,
        people[person]?.name ?? '',
        people[person]?.photo ?? '',
        people[person]?.notes ?? '',
        people[person]?.category ?? '',
        people[person]?.photographer_count ?? 0,
        people[person]?.photo_count ?? 0,
      );
    }
    return mapped;
  }

  public async setName(name: string) {
    this._name = name;
    await set_person_name(this.id, name);
  }

  public async setNotes(notes: string) {
    this._notes = notes;
    await set_person_notes(this.id, notes);
  }

  public async setCategory(category: string) {
    this._category = category;
    await set_person_category(this.id, category);
  }

  public async setPhoto(photo: string) {
    this._photo = photo;
    await set_person_photo(this.id, photo);
  }
}
