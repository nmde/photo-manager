import { Entity } from './Entity';

export type TagData = {
  name: string;
  color: string;
  prereqs: string;
  coreqs: string;
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
    return this.data.prereqs.length === 0 ? [] : this.data.prereqs.split(',');
  }

  public get coreqs() {
    return this.data.coreqs.length === 0 ? [] : this.data.coreqs.split(',');
  }

  public get incompatible() {
    return this.data.incompatible.length === 0 ? [] : this.data.incompatible.split(',');
  }

  public set prereqs(tags: string[]) {
    this.data.prereqs = tags.join(',');
  }

  public set coreqs(tags: string[]) {
    this.data.coreqs = tags.join(',');
  }

  public set incompatible(tags: string[]) {
    this.data.incompatible = tags.join(',');
  }
}
