import { Entity } from './Entity';

export interface PhotoData {
  name: string;
  path: string;
  title: string;
  description: string;
  location: string;
  tags: string;
  isDuplicate: boolean;
  thumbnail: string;
  rating: number;
  video: boolean;
  photoGroup: string;
  date: string;
  raw: boolean;
  people: string;
  hideThumbnail: boolean;
  photographer: string;
}

export class Photo extends Entity<PhotoData> {
  public awaitingThumbnail = true;

  public firstInGroup = false;

  public hidden = false;

  public rawFile = '';

  public valid = true;

  public validationMsg = '';

  public constructor(data: PhotoData) {
    super('Photo', data);
    this.primaryKey = 'name';
  }

  public get group() {
    if (this.data.photoGroup.length === 0) {
      return undefined;
    }
    return this.data.photoGroup;
  }

  public get hasLocation() {
    return typeof this.data.location === 'string' && this.data.location.length > 0;
  }

  public get tags() {
    if (this.data.tags.length === 0) {
      return [];
    }
    return this.data.tags.split(',');
  }

  public set tags(value: string[]) {
    this.data.tags = value.join(',');
  }

  public get rating() {
    if (this.hasRating) {
      return this.data.rating;
    }
  }

  /**
   * If the photo has a rating.
   */
  public get hasRating() {
    return typeof this.data.rating === 'number' && this.data.rating > 0;
  }

  /**
   * Checks if this photo has the specified tag.
   * @param tag - The tag to check for.
   * @returns If this photo has the specified tag.
   */
  public hasTag(tag: string) {
    return this.tags.indexOf(tag) >= 0;
  }

  public get date() {
    return new Date(this.data.date);
  }

  public get people() {
    if (this.data.people === null || this.data.people.length === 0) {
      return [];
    }
    return this.data.people.split(',');
  }

  public set people(people: string[]) {
    this.data.people = people.join(',');
  }
}

export function createPhoto(name: string, path: string): Photo {
  return new Photo({
    name,
    path,
    title: name,
    description: '',
    tags: '',
    isDuplicate: false,
    rating: 0,
    location: '',
    thumbnail: '',
    video: false,
    photoGroup: '',
    date: '',
    raw: false,
    people: '',
    hideThumbnail: false,
    photographer: '',
  });
}
