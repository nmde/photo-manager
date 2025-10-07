import { invoke } from '@tauri-apps/api/core';

/**
 * Table to store information about tags.
 */
export class Tag {
  public constructor(
    private _id: string,
    private _name: string,
    private _color: string,
    private _prereqs: string,
    private _coreqs: string,
    private _incompatible: string,
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
    return this._prereqs.length === 0 ? [] : this._prereqs.split(',');
  }

  public get coreqs() {
    return this._coreqs.length === 0 ? [] : this._coreqs.split(',');
  }

  public get incompatible() {
    return this._incompatible.length === 0 ? [] : this._incompatible.split(',');
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
    this._prereqs = tags.join(',');
    await invoke('set_tag_str', {
      tag: this.id,
      property: 'prereqs',
      value: this._prereqs,
    });
  }

  public async setCoreqs(tags: string[]) {
    this._coreqs = tags.join(',');
    await invoke('set_tag_str', {
      tag: this.id,
      property: 'coreqs',
      value: this._coreqs,
    });
  }

  public async setIncompatible(tags: string[]) {
    this._incompatible = tags.join(',');
    await invoke('set_tag_str', {
      tag: this.id,
      property: 'incompatible',
      value: this._incompatible,
    });
  }
}
