import { Entity } from './Entity';

type PersonCategoryData = {
  name: string;
  color: string;
};

export class PersonCategory extends Entity<PersonCategoryData> {
  public constructor(data: PersonCategoryData) {
    super('PersonCategory', data);
  }
}
