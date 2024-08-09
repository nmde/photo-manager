import { Entity } from './Entity';

type PersonData = {
  name: string;
  photo: string;
  notes: string;
  category: string;
};

export class Person extends Entity<PersonData> {
  public constructor(data: PersonData) {
    super('Person', data);
  }
}
