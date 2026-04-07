import { set_tag_color, set_tag_coreqs, set_tag_incompatible, set_tag_prereqs } from '@/api/tags';

export type TagData = {
  name: string;
  color: string;
  prereqs: string[];
  coreqs: string[];
  incompatible: string[];
  count: number;
};

/**
 * Table to store information about tags.
 */
export class Tag {
  public constructor(
    public readonly name: string,
    public _color: string,
    public _prereqs: string[],
    public _coreqs: string[],
    public _incompatible: string[],
    public count: number,
  ) {}

  public get id() {
    return this.name;
  }

  public get color() {
    return this._color;
  }

  public get prereqs() {
    return this._prereqs;
  }

  public get coreqs() {
    return this._coreqs;
  }

  public get incompatible() {
    return this._incompatible;
  }

  public static createTags = (data: TagData[]) => {
    const tags: Record<string, Tag> = {};
    for (const tag of data) {
      tags[tag.name] = new Tag(
        tag.name,
        tag.color,
        tag.prereqs,
        tag.coreqs,
        tag.incompatible,
        tag.count,
      );
    }
    return tags;
  };

  public static default = (name?: string) => new Tag(name ?? '', '', [], [], [], 0);

  public async setColor(color: string) {
    this._color = color;
    await set_tag_color(this.name, color);
  }

  public async setPrereqs(tags: string[]) {
    this._prereqs = tags;
    await set_tag_prereqs(this.name, tags);
  }

  public async setCoreqs(tags: string[]) {
    this._coreqs = tags;
    await set_tag_coreqs(this.name, tags);
  }

  public async setIncompatible(tags: string[]) {
    this._incompatible = tags;
    await set_tag_incompatible(this.name, tags);
  }
}
