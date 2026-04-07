import { set_person_category, set_person_name, set_person_photo } from '@/api/people';

export type PersonData = {
  id: string;
  name: string;
  photo: string;
  category: string;
  photographer_count: number;
  photo_count: number;
};

export class Person {
  public constructor(
    public readonly id: string,
    private _name: string,
    private _photo: string,
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

  public get category() {
    return this._category;
  }

  public static createPeople(people: PersonData[]) {
    const mapped: Record<string, Person> = {};
    for (const person of people) {
      mapped[person.id] = new Person(
        person.id,
        person.name,
        person.photo,
        person.category,
        person.photographer_count,
        person.photo_count,
      );
    }
    return mapped;
  }

  public async setName(name: string) {
    this._name = name;
    await set_person_name(this.id, name);
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
