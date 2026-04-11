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
import { validate_photo, type ValidationResult } from '@/api/tags';

export type PhotoData = {
  name: string;
  asset_path: string;
  title: string | null;
  description: string | null;
  tags: string[];
  is_duplicate: boolean;
  rating: number | null;
  is_video: boolean;
  location: string | null;
  thumbnail: string | null;
  photo_group: string | null;
  date: string | null;
  is_raw: boolean;
  people: string[];
  hide_thumbnail: boolean;
  photographer: string | null;
  valid_tags: ValidationResult;
  metadata_date: string | null;
  metadata_location: [number, number] | null;
};

// The _variables here have to be public or eslint complains about them being used in vue components
export class Photo {
  public _date: Date | null = null;
  public _metaDate: Date | null = null;

  public constructor(
    public readonly name: PhotoData['name'],
    public readonly asset_path: PhotoData['asset_path'],
    public _title: PhotoData['title'],
    public _description: PhotoData['description'],
    public _location: PhotoData['location'],
    public _tags: PhotoData['tags'],
    public _isDuplicate: PhotoData['is_duplicate'],
    public readonly thumbnail: PhotoData['thumbnail'],
    public _rating: PhotoData['rating'],
    public readonly is_video: PhotoData['is_video'],
    public _photoGroup: PhotoData['photo_group'],
    date: PhotoData['date'],
    public readonly is_raw: PhotoData['is_raw'],
    public _people: PhotoData['people'],
    public _hideThumbnail: PhotoData['hide_thumbnail'],
    private _photographer: PhotoData['photographer'],
    public valid_tags: PhotoData['valid_tags'],
    public readonly metadata_date: PhotoData['metadata_date'],
    public readonly metadata_location: PhotoData['metadata_location'],
  ) {
    if (date !== null && date.length > 0) {
      this._date = this.parseDate(date);
    }
    if (metadata_date !== null && metadata_date.length > 0) {
      this._metaDate = this.parseDate(metadata_date);
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
    return this._rating;
  }

  public get hideThumbnail() {
    return this._hideThumbnail;
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

  public get metaDate() {
    return this._metaDate;
  }

  public static createPhotos = (data: PhotoData[]) =>
    data.map(
      ({
        name,
        asset_path,
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
        metadata_date,
        metadata_location,
      }) =>
        new Photo(
          name,
          asset_path,
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
          metadata_date,
          metadata_location,
        ),
    );

  public static default = () =>
    new Photo(
      '',
      '',
      null,
      null,
      null,
      [],
      false,
      null,
      null,
      false,
      null,
      null,
      false,
      [],
      false,
      null,
      { is_valid: true, message: null },
      null,
      null,
    );

  public async setTitle(value: string | null) {
    this._title = value;
    await set_photo_title(this.name, value);
  }

  public async setDescription(value: string | null) {
    this._description = value;
    await set_photo_desc(this.name, value);
  }

  public async setLocation(value: string | null) {
    this._location = value;
    await set_photo_location(this.name, value);
  }

  public async setTags(value: string[]) {
    this._tags = value;
    await set_photo_tags(this.name, value)
      .err(msg => reportError(msg))
      .send();
    await validate_photo(this.name)
      .ok(async validation => {
        this.setValidation(validation);
      })
      .err(msg => reportError(msg))
      .send();
  }

  public async setDuplicate(value: boolean) {
    this._isDuplicate = value;
    await set_photo_is_duplicate(this.name, value);
  }

  public async setRating(rating: number | null) {
    this._rating = rating;
    await set_photo_rating(this.name, rating);
  }

  public async setDate(value: Date | null) {
    this._date = value;
    await set_photo_date(this.name, value ? value.toISOString().slice(0, 10) : '');
  }

  public async setPeople(people: string[]) {
    this._people = people;
    await set_photo_people(this.name, people);
  }

  public async setHideThumbnail(value: boolean) {
    this._hideThumbnail = value;
    await set_photo_hide_thumbnail(this.name, value);
  }

  public async setPhotographer(value: string | null) {
    this._photographer = value;
    await set_photographer(this.name, value);
  }

  public async setGroup(value: string | null) {
    this._photoGroup = value;
    await set_photo_group(this.name, value);
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
    this.valid_tags = validation;
  }

  private parseDate(str: string) {
    const split = str.split('-').map(part => Number.parseInt(part));
    if (split.length !== 3) {
      throw new Error('Malformed date string');
    }
    const split2 = split as [number, number, number];
    return new Date(split2[0], split2[1] - 1, split[2]);
  }
}
