import { Photo } from "../classes/Photo";

export type PhotoDataFile = {
    files: Record<string, Photo>;
    locations: Record<string, string>;
    tags: string[];
};
