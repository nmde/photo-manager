import { convertFileSrc } from '@tauri-apps/api/tauri';

/**
 * Represents information about a photo.
 */
export class Photo {
    public path: string;

    /**
     * Constructs Photo.
     * @param name - The file name.
     * @param path - The file path.
     */
    public constructor(public name: string, path: string) {
        this.path = convertFileSrc(path);
    }
}
