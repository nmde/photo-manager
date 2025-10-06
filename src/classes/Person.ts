import { Entity } from './Entity';

type PersonData = {
  name: string;
  photo: string;
  notes: string;
  category: string;
};

export class Person extends Entity<PersonData> {
  public count = 0;

  public photographerCount = 0;

  public constructor(data: PersonData) {
    super('Person', data);
  }
}
