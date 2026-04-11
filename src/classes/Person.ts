import type { PersonCategoryData } from './PersonCategory';
import type { Nullable } from '@/types';
import { set_person_category, set_person_name, set_person_photo } from '@/api/people';
import { SortableItem } from './SortableItem';

export type PersonData = {
  id: string;
  name: string;
  photo: Nullable<string>;
  category: PersonCategoryData['id'];
  photographer_count: number;
  photo_count: number;
};

export type PersonRec = Record<PersonData['id'], Person>;

export class Person extends SortableItem implements PersonData {
  public constructor(
    public readonly id: PersonData['id'],
    _name: PersonData['name'],
    _photo: PersonData['photo'],
    private _category: PersonData['category'],
    public photographer_count: PersonData['photographer_count'],
    public photo_count: PersonData['photo_count'],
  ) {
    super(id, photo_count, _name, _photo);
  }

  public get category() {
    return this._category;
  }

  public static createPeople(people: PersonData[]) {
    const mapped: PersonRec = {};
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

  public async setName(name: PersonData['name']) {
    this._name = name;
    await set_person_name(this.id, name);
  }

  public async setCategory(category: PersonData['category']) {
    this._category = category;
    await set_person_category(this.id, category);
  }

  public async setPhoto(photo: PersonData['photo']) {
    this._photo = photo;
    await set_person_photo(this.id, photo);
  }
}
