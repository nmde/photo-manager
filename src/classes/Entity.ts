import { v4 as uuid } from 'uuid';

export class Entity<T = any> {
  public Id: string;

  public primaryKey?: string;

  public constructor(
    public tableName: string,
    public data: T,
  ) {
    this.Id = uuid();
  }
}
