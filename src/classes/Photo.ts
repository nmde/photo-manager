import { convertFileSrc } from '@tauri-apps/api/tauri';

export interface Photo {
    name: string;
    path: string;
    title: string;
    description: string;
    location?: {
        lat: number;
        lng: number;
    };
    locationApprox: boolean;
    tags: string[];
    isDuplicate: boolean;
    thumbnail?: string;
    rating?: number;
    video?: boolean;
    group?: string;
};

export function createPhoto(name: string, path: string): Photo {
    return {
        name,
        path: convertFileSrc(path),
        title: name,
        description: '',
        locationApprox: false,
        tags: [],
        isDuplicate: false,
    };
}
