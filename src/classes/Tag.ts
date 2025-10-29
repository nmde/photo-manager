import { invoke } from '@tauri-apps/api/core';

export type TagData = {
  id: string;
  name: string;
  color: string;
  prereqs: string[];
  coreqs: string[];
  incompatible: string[];
};

/**
 * Table to store information about tags.
 */
export class Tag {
  public constructor(
    private _id: string,
    private _name: string,
    private _color: string,
    private _prereqs: string[],
    private _coreqs: string[],
    private _incompatible: string[],
  ) {}

  public get id() {
    return this._id;
  }

  public get name() {
    return this._name;
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

  public static createTags(data: TagData[]) {
    return data.map(
      ({ id, name, color, prereqs, coreqs, incompatible }) =>
        new Tag(id, name, color, prereqs, coreqs, incompatible),
    );
  }

  public async setColor(color: string) {
    this._color = color;
    await invoke('set_tag_str', {
      tag: this.id,
      property: 'color',
      value: color,
    });
  }

  public async setPrereqs(tags: string[]) {
    this._prereqs = tags;
    await invoke('set_tag_str', {
      tag: this.id,
      property: 'prereqs',
      value: this._prereqs,
    });
  }

  public async setCoreqs(tags: string[]) {
    this._coreqs = tags;
    await invoke('set_tag_str', {
      tag: this.id,
      property: 'coreqs',
      value: this._coreqs,
    });
  }

  public async setIncompatible(tags: string[]) {
    this._incompatible = tags;
    await invoke('set_tag_str', {
      tag: this.id,
      property: 'incompatible',
      value: this._incompatible,
    });
  }
}
