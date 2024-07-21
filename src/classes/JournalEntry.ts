import { Entity } from './Entity';

type JournalData = {
  date: string;
  mood: number;
  text: string;
};

export class JournalEntry extends Entity<JournalData> {
  public constructor(data: JournalData) {
    super('Journal', data);
  }

  public get date() {
    return new Date(this.data.date);
  }
}
