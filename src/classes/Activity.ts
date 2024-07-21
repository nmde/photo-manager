import { Entity } from './Entity';

type ActivityData = {
  icon: string;
  name: string;
};

export class Activity extends Entity<ActivityData> {
  public constructor(data: ActivityData) {
    super('Activity', data);
  }
}
