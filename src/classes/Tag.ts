import { Entity } from './Entity';

export type TagData = {
  name: string;
  color: string;
  prereqs: string;
  incompatible: string;
};

/**
 * Table to store information about tags.
 */
export class Tag extends Entity<TagData> {
  public constructor(data: TagData) {
    super('Tag', data);
    this.primaryKey = 'name';
  }

  public get prereqs() {
    if (this.data.prereqs.length === 0) {
      return [];
    }
    return this.data.prereqs.split(',');
  }

  public set prereqs(tags: string[]) {
    this.data.prereqs = tags.join(',');
  }

  public get incompatible() {
    if (this.data.incompatible.length === 0) {
      return [];
    }
    return this.data.incompatible.split(',');
  }

  public set incompatible(tags: string[]) {
    this.data.incompatible = tags.join(',');
  }
}
