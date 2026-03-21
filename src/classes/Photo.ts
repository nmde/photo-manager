import {
  set_photo_bool,
  set_photo_date,
  set_photo_location,
  set_photo_people,
  set_photo_rating,
  set_photo_str,
  set_photo_tags,
  set_photographer,
} from '@/api/photos';
import type { ValidationResult } from '@/api/tags';

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
  date: [number, number] | null;
  raw: number;
  people: string[];
  hide_thumbnail: number;
  photographer: string;
  valid_tags: boolean;
  validation_msg: string;
};

// The _variables here have to be public or eslint complains about them being used in vue components
export class Photo {
  public readonly video: boolean;
  public readonly raw: boolean;
  public _date?: Date;

  public constructor(
    public readonly id: string,
    public readonly name: string,
    public readonly path: string,
    public _title: string,
    public _description: string,
    public _location: string,
    public _tags: string[],
    public _isDuplicate: boolean,
    public readonly thumbnail: string,
    public _rating: number,
    _video: boolean,
    public _photoGroup: string,
    date: [number, number] | null,
    _raw: boolean,
    public _people: string[],
    public _hideThumbnail: boolean,
    private _photographer: string,
    public valid: boolean,
    public validationMsg: string,
  ) {
    this.video = _video;
    this.raw = _raw;
    if (date !== null) {
      this._date = new Date(date[0], 0, 0);
      this._date.setDate(this._date.getDate() + date[1]);
    }
  }

  public get title() {
    return this._title;
  }

  public get description() {
    return this._description;
  }

  public get group() {
    return this._photoGroup.length === 0 ? undefined : this._photoGroup;
  }

  public get hasLocation() {
    return this._location.length > 0;
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
          valid_tags,
          validation_msg,
        ),
    );

  public static default = () =>
    new Photo(
      '',
      '',
      '',
      '',
      '',
      '',
      [],
      false,
      '',
      0,
      false,
      '',
      [0, 0],
      false,
      [],
      false,
      '',
      true,
      '',
    );

  public async setTitle(value: string) {
    this._title = value;
    await set_photo_str(this.id, 'title', value);
  }

  public async setDescription(value: string) {
    this._description = value;
    await set_photo_str(this.id, 'description', value);
  }

  public async setLocation(value: string) {
    this._location = value;
    await set_photo_location(this.id, value);
  }

  public async setTags(value: string[]) {
    this._tags = value;
    return await set_photo_tags(this.id, value);
  }

  public async setDuplicate(value: boolean) {
    this._isDuplicate = value;
    await set_photo_bool(this.id, 'isDuplicate', value);
  }

  public async setRating(rating: number) {
    this._rating = rating;
    await set_photo_rating(this.id, rating);
  }

  public async setDate(value?: Date) {
    this._date = value;
    await set_photo_date(this.id, value ? value.toISOString().slice(0, 10) : '');
  }

  public async setPeople(people: string[]) {
    this._people = people;
    await set_photo_people(this.id, people);
  }

  public async setHideThumbnail(value: boolean) {
    this._hideThumbnail = value;
    await set_photo_bool(this.id, 'hideThumbnail', value);
  }

  public async setPhotographer(value: string) {
    this._photographer = value;
    await set_photographer(this.id, value);
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
