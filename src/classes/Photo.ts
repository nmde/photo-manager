import { convertFileSrc } from '@tauri-apps/api/tauri';
import { Entity } from './Entity';
import { type Position } from './Map';

export interface PhotoData {
  name: string;
  path: string;
  title: string;
  description: string;
  location: string;
  locationApprox: boolean;
  tags: string;
  isDuplicate: boolean;
  thumbnail: string;
  rating: number;
  video: boolean;
  photoGroup: string;
  date: string;
  raw: boolean;
}

export class Photo extends Entity<PhotoData> {
  public awaitingThumbnail = true;

  public firstInGroup = false;

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

  public get location(): Position | undefined {
    if (this.data.location) {
      return JSON.parse(this.data.location);
    }
    return undefined;
  }

  public set location(location: Position) {
    this.data.location = JSON.stringify(location);
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
    if (typeof this.data.rating === 'number') {
      return this.data.rating;
    }
  }

  /**
   * Checks if this photo has the specified tag.
   * @param tag - The tag to check for.
   * @returns If this photo has the specified tag.
   */
  public hasTag(tag: string) {
    return this.tags.indexOf(tag) >= 0;
  }
}

export function createPhoto(name: string, path: string): Photo {
  return new Photo({
    name,
    path: convertFileSrc(path),
    title: name,
    description: '',
    locationApprox: false,
    tags: '',
    isDuplicate: false,
    rating: 0,
    location: '',
    thumbnail: '',
    video: false,
    photoGroup: '',
    date: '',
    raw: false,
  });
}
