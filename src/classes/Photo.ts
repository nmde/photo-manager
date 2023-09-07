import { convertFileSrc } from '@tauri-apps/api/tauri';
import { Entity } from './Entity';

export interface PhotoData {
  name: string;
  path: string;
  title: string;
  description: string;
  location?: string;
  locationApprox: boolean;
  tags: string;
  isDuplicate: boolean;
  thumbnail?: string;
  rating?: number;
  video?: boolean;
  group?: string;
}

export class Photo extends Entity<PhotoData> {
  public constructor(data: PhotoData) {
    super('Photo', data);
  }

  public get location() {
    if (this.data.location) {
      return JSON.parse(this.data.location);
    }
    return {
      lat: 0,
      lng: 0,
    };
  }

  public set location(location: { lat: number; lng: number }) {
    this.data.location = JSON.stringify(location);
  }

  public get tags() {
    return this.data.tags.split(',');
  }

  public set tags(value: string[]) {
    this.data.tags = value.join(',');
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
  });
}
