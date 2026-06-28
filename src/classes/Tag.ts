import type { Nullable } from '@/types';
import { set_tag_color, set_tag_coreqs, set_tag_incompatible, set_tag_prereqs } from '@/api/tags';
import { SortableItem } from './SortableItem';

export type TagData = {
  name: string;
  color: Nullable<string>;
  prereqs: string[];
  coreqs: string[];
  incompatible: string[];
  count: number;
};

export type TagRec = Record<TagData['name'], Tag>;

/**
 * Table to store information about tags.
 */
export class Tag extends SortableItem implements TagData {
  public constructor(
    public readonly _name: TagData['name'],
    public _color: TagData['color'],
    public _prereqs: TagData['prereqs'],
    public _coreqs: TagData['coreqs'],
    public _incompatible: TagData['incompatible'],
    public count: TagData['count'],
  ) {
    super(_name, count, _name, null);
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
    const tags: TagRec = {};
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

  public static default = (name?: TagData['name']) => new Tag(name ?? '', '', [], [], [], 0);

  public async setColor(color: TagData['color']) {
    this._color = color;
    await set_tag_color(this.name, color);
  }

  public async setPrereqs(tags: TagData['prereqs']) {
    this._prereqs = tags;
    await set_tag_prereqs(this.name, tags);
  }

  public async setCoreqs(tags: TagData['coreqs']) {
    this._coreqs = tags;
    await set_tag_coreqs(this.name, tags);
  }

  public async setIncompatible(tags: TagData['incompatible']) {
    this._incompatible = tags;
    await set_tag_incompatible(this.name, tags);
  }
}
