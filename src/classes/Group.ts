import { Entity } from './Entity';

type GroupData = {
  name: string;
};

export class Group extends Entity<GroupData> {
  public constructor(data: GroupData) {
    super('PhotoGroup', data);
  }
}
