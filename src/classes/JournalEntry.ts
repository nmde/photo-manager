import type { Activity } from './Activity';
import { Entity } from './Entity';

type JournalData = {
  date: string;
  mood: number;
  text: string;
  activities: string;
  steps: number;
  iv: string;
};

export class JournalEntry extends Entity<JournalData> {
  public activities: Activity[] = [];

  public constructor(data: JournalData) {
    super('Journal', data);
  }

  public get date() {
    return new Date(this.data.date);
  }
}
