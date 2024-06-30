import { Entity } from './Entity';

type JournalData = {};

export class Journal extends Entity<JournalData> {
  public constructor(data: JournalData) {
    super('Journal', data);
  }
}
