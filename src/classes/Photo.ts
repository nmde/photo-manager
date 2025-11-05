import { invoke } from '@tauri-apps/api/core';

export type PhotoData = {
  id: string;
  name: string;
  path: string;
  title: string;
  description: string;
  tags: string[];
  is_duplicate: number;
  rating: number;
  location: string;
  thumbnail: string;
  video: number;
  photo_group: string;
  date: string;
  raw: number;
  people: string[];
  hide_thumbnail: number;
  photographer: string;
  camera: string;
  valid_tags: boolean;
  validation_msg: string;
};

export class Photo {
  public awaitingThumbnail = true;

  public firstInGroup = false;

  public hidden = false;

  public rawFile = '';

  public validationMsg = '';

  public constructor(
    private _id: string,
    private _name: string,
    private _path: string,
    private _title: string,
    private _description: string,
    private _location: string,
    private _tags: string[],
    private _isDuplicate: boolean,
    private _thumbnail: string,
    private _rating: number,
    private _video: boolean,
    private photoGroup: string,
    private _date: string,
    private _raw: boolean,
    private _people: string[],
    private _hideThumbnail: boolean,
    private _photographer: string,
    private _camera: string,
    public valid: boolean,
    public validationMessage: string,
  ) {}

  public get id() {
    return this._id;
  }

  public get name() {
    return this._name;
  }

  public get path() {
    return this._path;
  }

  public get title() {
    return this._title;
  }

  public get description() {
    return this._description;
  }

  public get group() {
    return this.photoGroup.length === 0 ? undefined : this.photoGroup;
  }

  public get hasLocation() {
    return typeof this._location === 'string' && this._location.length > 0;
  }

  public get location() {
    return this._location;
  }

  public get tags() {
    return this._tags;
  }

  public get isDuplicate() {
    return this._isDuplicate;
  }

  public get thumbnail() {
    return this._thumbnail;
  }

  public get rating() {
    if (this.hasRating) {
      return this._rating;
    }
  }

  public get video() {
    return this._video;
  }

  public get hideThumbnail() {
    return this._hideThumbnail;
  }

  /**
   * If the photo has a rating.
   */
  public get hasRating() {
    return typeof this._rating === 'number' && this._rating > 0;
  }

  public get hasDate() {
    return this._date.length > 0;
  }

  public get date() {
    return new Date(this._date);
  }

  public get raw() {
    return this._raw;
  }

  public get people() {
    return this._people;
  }

  public get photographer() {
    return this._photographer;
  }

  public get camera() {
    return this._camera;
  }

  public static createPhotos(data: PhotoData[]) {
    return data.map(
      ({
        id,
        name,
        path,
        title,
        description,
        tags,
        is_duplicate,
        rating,
        location,
        thumbnail,
        video,
        photo_group,
        date,
        raw,
        people,
        hide_thumbnail,
        photographer,
        camera,
        valid_tags,
        validation_msg,
      }) =>
        new Photo(
          id,
          name,
          path,
          title,
          description,
          location,
          tags,
          is_duplicate === 1,
          thumbnail,
          rating,
          video === 1,
          photo_group,
          date,
          raw === 1,
          people,
          hide_thumbnail === 1,
          photographer,
          camera,
          valid_tags,
          validation_msg,
        ),
    );
  }

  public async setTitle(value: string) {
    this._title = value;
    await invoke('set_photo_str', {
      photo: this._id,
      property: 'title',
      value,
    });
  }

  public async setDescription(value: string) {
    this._description = value;
    await invoke('set_photo_str', {
      photo: this._id,
      property: 'description',
      value,
    });
  }

  public async setLocation(value: string) {
    this._location = value;
    await invoke('set_photo_location', {
      photo: this._id,
      value,
    });
  }

  public async setTags(value: string[]) {
    this._tags = value;
    return await invoke<{ is_valid: boolean; message: string }>('set_photo_tags', {
      photo: this._id,
      value,
    });
  }

  public async setDuplicate(value: boolean) {
    this._isDuplicate = value;
    await invoke('set_photo_bool', {
      photo: this._id,
      property: 'isDuplicate',
      value,
    });
  }

  public async setRating(rating: number) {
    this._rating = rating;
    await invoke('set_photo_rating', {
      photo: this._id,
      rating,
    });
  }

  public async setThumbnail(value: string) {
    this._thumbnail = value;
    await invoke('set_photo_str', {
      photo: this._id,
      property: 'thumbnail',
      value,
    });
  }

  public async setGroup(group: string) {
    this.photoGroup = group;
    await invoke('set_photo_group', {
      photo: this._id,
      value: group,
    });
  }

  public async setDate(value: string) {
    this._date = value;
    await invoke('set_photo_date', {
      photo: this._id,
      value,
    });
  }

  public async setRaw(value: boolean) {
    this._raw = value;
    await invoke('set_photo_bool', {
      photo: this._id,
      property: 'raw',
      value,
    });
  }

  public async setPeople(people: string[]) {
    this._people = people;
    await invoke('set_photo_people', {
      photo: this._id,
      value: this._people,
    });
  }

  public async setHideThumbnail(value: boolean) {
    this._hideThumbnail = value;
    await invoke('set_photo_bool', {
      photo: this._id,
      property: 'hideThumbnail',
      value,
    });
  }

  public async setPhotographer(value: string) {
    this._photographer = value;
    await invoke('set_photographer', {
      photo: this._id,
      value,
    });
  }

  public async setVideo(value: boolean) {
    this._video = value;
    await invoke('set_photo_bool', {
      photo: this._id,
      property: 'video',
      value,
    });
  }

  public async setCamera(value: string) {
    this._camera = value;
    await invoke('set_photo_camera', {
      photo: this._id,
      value,
    });
  }

  /**
   * Checks if this photo has the specified tag.
   * @param tag - The tag to check for.
   * @returns If this photo has the specified tag.
   */
  public hasTag(tag: string) {
    return this.tags.includes(tag);
  }
}
