import { v4 as uuid } from 'uuid';

export class Entity<T = any> {
  public id: string;

  public constructor(
    public tableName: string,
    public data: T,
  ) {
    this.id = uuid();
  }
}
