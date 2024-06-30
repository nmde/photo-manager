import { Entity } from './Entity';

type JournalData = {};

export class JournalEntry extends Entity<JournalData> {
  public constructor(data: JournalData) {
    super('Journal', data);
  }
}
