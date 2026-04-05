import type { ValidationResult } from '@/api/tags';
import {
  set_photo_date,
  set_photo_desc,
  set_photo_group,
  set_photo_hide_thumbnail,
  set_photo_is_duplicate,
  set_photo_location,
  set_photo_people,
  set_photo_rating,
  set_photo_tags,
  set_photo_title,
  set_photographer,
} from '@/api/photos';

export type PhotoData = {
  path: string;
  title?: string;
  description?: string;
  tags: string[];
  is_duplicate: boolean;
  rating?: number;
  location?: string;
  thumbnail?: string;
  is_video: boolean;
  photo_group?: string;
  date?: string;
  is_raw: boolean;
  people: string[];
  hide_thumbnail: boolean;
  photographer?: string;
  valid_tags: boolean;
  validation_msg: string;
};

// The _variables here have to be public or eslint complains about them being used in vue components
export class Photo {
  public _date?: Date;

  public constructor(
    public readonly path: string,
    public _title: string | undefined,
    public _description: string | undefined,
    public _location: string | undefined,
    public _tags: string[],
    public _isDuplicate: boolean,
    public readonly thumbnail: string | undefined,
    public _rating: number | undefined,
    public readonly is_video: boolean,
    public _photoGroup: string | undefined,
    date: string | undefined,
    public readonly is_raw: boolean,
    public _people: string[],
    public _hideThumbnail: boolean,
    private _photographer: string | undefined,
    public valid: boolean,
    public validationMsg: string,
  ) {
    if (date !== undefined && date.length > 0) {
      const split = date.split('-').map(part => Number.parseInt(part)) as [number, number, number];
      this._date = new Date(split[0], split[1] - 1, split[2]);
    }
  }

  public get title() {
    return this._title;
  }

  public get description() {
    return this._description;
  }

  public get group() {
    return this._photoGroup;
  }

  public get hasLocation() {
    return this._location !== undefined;
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

  public get rating() {
    if (this.hasRating) {
      return this._rating;
    }
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

  public get date() {
    return this._date;
  }

  public get people() {
    return this._people;
  }

  public get photographer() {
    return this._photographer;
  }

  public static createPhotos = (data: PhotoData[]) =>
    data.map(
      ({
        path,
        title,
        description,
        tags,
        is_duplicate,
        rating,
        location,
        thumbnail,
        is_video,
        photo_group,
        date,
        is_raw,
        people,
        hide_thumbnail,
        photographer,
        valid_tags,
        validation_msg,
      }) =>
        new Photo(
          path,
          title,
          description,
          location,
          tags,
          is_duplicate,
          thumbnail,
          rating,
          is_video,
          photo_group,
          date,
          is_raw,
          people,
          hide_thumbnail,
          photographer,
          valid_tags,
          validation_msg,
        ),
    );

  public static default = () =>
    new Photo(
      '',
      undefined,
      undefined,
      undefined,
      [],
      false,
      undefined,
      undefined,
      false,
      undefined,
      undefined,
      false,
      [],
      false,
      undefined,
      true,
      '',
    );

  public async setTitle(value: string) {
    this._title = value;
    await set_photo_title(this.path, value);
  }

  public async setDescription(value: string) {
    this._description = value;
    await set_photo_desc(this.path, value);
  }

  public async setLocation(value: string) {
    this._location = value;
    await set_photo_location(this.path, value);
  }

  public async setTags(value: string[]) {
    this._tags = value;
    return await set_photo_tags(this.path, value);
  }

  public async setDuplicate(value: boolean) {
    this._isDuplicate = value;
    await set_photo_is_duplicate(this.path, value);
  }

  public async setRating(rating: number) {
    this._rating = rating;
    await set_photo_rating(this.path, rating);
  }

  public async setDate(value?: Date) {
    this._date = value;
    await set_photo_date(this.path, value ? value.toISOString().slice(0, 10) : '');
  }

  public async setPeople(people: string[]) {
    this._people = people;
    await set_photo_people(this.path, people);
  }

  public async setHideThumbnail(value: boolean) {
    this._hideThumbnail = value;
    await set_photo_hide_thumbnail(this.path, value);
  }

  public async setPhotographer(value: string) {
    this._photographer = value;
    await set_photographer(this.path, value);
  }

  public async setGroup(value: string) {
    this._photoGroup = value;
    await set_photo_group(this.path, value);
  }

  /**
   * Checks if this photo has the specified tag.
   * @param tag - The tag to check for.
   * @returns If this photo has the specified tag.
   */
  public hasTag(tag: string) {
    return this.tags.includes(tag);
  }

  public setValidation(validation: ValidationResult) {
    this.valid = validation.is_valid;
    this.validationMsg = validation.message;
  }
}
