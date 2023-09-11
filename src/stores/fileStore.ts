import { defineStore } from 'pinia';
import { computed, ref } from 'vue';
import { locToString } from '../classes/Map';
import { Group } from '../classes/Group';
import { createPhoto, Photo } from '../classes/Photo';
import { TauriDatabase } from '@/classes/TauriDatabase';

export const useFileStore = defineStore('files', () => {
  let database: TauriDatabase | null = null;

  const files = ref<Record<string, Photo>>({});

  const groups = ref<Group[]>([]);

  const workingDir = ref('');

  const tags = ref<string[]>([]);

  const tagCounts = ref<Record<string, number>>({});

  const locations = computed(() => {
    const locRecord: Record<string, number> = {};
    Object.values(files.value).forEach((file) => {
      if (file.data.location !== undefined) {
        const key = locToString(file.location);
        if (!locRecord[key]) {
          locRecord[key] = 0;
        }
        locRecord[key] += 1;
      }
    });
    return locRecord;
  });

  /**
   * Adds a file to the registry.
   * @param file - The file to add.
   */
  async function addFile(file: any) {
    if (typeof file.name === 'string') {
      files.value[file.name] = createPhoto(file.name, file.path);
    } else {
      throw new Error(`Unexpected file: ${file.path}`);
    }
  }

  /**
   * Sets the working dir name.
   * @param path - The path to the working dir.
   */
  async function setWorkingDir(path: string) {
    workingDir.value = path;
    const { join } = await import('@tauri-apps/api/path');
    database = new TauriDatabase(`sqlite:${await join(path, 'photos.db')}`);
  }

  /**
   * Sets the stored photo data for a file.
   * @param name - The name of the file to set.
   * @param photo - The data to set.
   */
  async function setPhotoData(name: string, photo: Photo) {
    files.value[name] = photo;
    await database?.insert(photo);
    if (photo.group) {
      setGroup(name, photo.group);
    }
    updateTags(name, photo.tags);
    photo.tags.forEach((tag) => {
      if (!tagCounts.value[tag]) {
        tagCounts.value[tag] = 0;
      }
      tagCounts.value[tag] += 1;
    });
  }

  /**
   * Sets a photo's thumbnail property.
   * @param photo - The photo to set for.
   * @param thumbnail - The path to the thumbnail.
   */
  function setThumbnail(photo: string, thumbnail: string) {
    files.value[photo].data.thumbnail = thumbnail;
  }

  const photoCount = computed(() => {
    return Object.values(files.value).length;
  });

  /**
   * Sets the photo's location data.
   * @param photo - The target photo.
   * @param location - The location.
   */
  async function setLocation(photo: string, location: { lat: number; lng: number }) {
    files.value[photo].location = location;
    await database?.insert(files.value[photo]);
  }

  /**
   * Moves tags to the front of the list.
   * @param tags - The tags to move to the front.
   */
  function moveTagsToFront(targets: string[]) {
    targets.forEach((tag) => {
      tags.value.splice(tags.value.indexOf(tag), 1);
    });
    tags.value.unshift(...targets);
  }

  /**
   * Marks the photo as a video.
   * @param photo - The vidoe.
   */
  async function setVideo(photo: string) {
    files.value[photo].data.video = true;
    await database?.insert(files.value[photo]);
  }

  /**
   * Adds a group.
   * @param name - The name of the group.
   */
  async function addGroup(name: string) {
    const g = new Group({ name });
    groups.value.push(g);
    await database?.insert(g);
  }

  /**
   * Gets a list of group names.
   */
  const groupNames = computed(() => {
    return groups.value.map((g) => g.data.name).reverse();
  });

  /**
   * Sets a photo's rating.
   * @param photo - The photo to set for.
   * @param rating - The rating to set.
   */
  async function setRating(photo: string, rating: number) {
    files.value[photo].data.rating = rating;
    await database?.insert(files.value[photo]);
  }

  /**
   * Sets a photo's isDuplicate marker.
   * @param photo - The photo to set for.
   * @param isDuplicate - The duplicate marker.
   */
  async function setDuplicate(photo: string, isDuplicate: boolean) {
    files.value[photo].data.isDuplicate = isDuplicate;
    await database?.insert(files.value[photo]);
  }

  /**
   * Gets all photos in a group.
   * @param group - The group to get photos from.
   */
  function getByGroup(group: string) {
    return Object.values(files.value).filter((p) => p.data.photoGroup === group);
  }

  /**
   * Sets a photo's group.
   * @param photo - The photo to set.
   * @param group - The group to set.
   */
  async function setGroup(photo: string, group?: string) {
    if (group === undefined) {
      files.value[photo].data.photoGroup = '';
      return;
    }
    files.value[photo].data.photoGroup = group;
    await database?.insert(files.value[photo]);
    const collectedTags: string[] = [];
    getByGroup(group).forEach((photo) => {
      files.value[photo.data.name].tags.forEach((tag) => {
        if (collectedTags.indexOf(tag) < 0) {
          collectedTags.push(tag);
        }
      });
    });
    updateTags(photo, collectedTags);
  }

  /**
   * Removes a photo from its group.
   * @param photo - The photo to remove from its group.
   */
  async function removeGroup(photo: string) {
    files.value[photo].data.photoGroup = '';
    await database?.insert(files.value[photo]);
  }

  /**
   * Adds new tags to the master list.
   * @param photo - The photo to apply tags to.
   * @param t - The tags to apply.
   */
  async function updateTags(photo: string, t: string[]) {
    t.forEach((tag) => {
      if (!tagCounts.value[tag]) {
        tagCounts.value[tag] = 0;
      }
      if (files.value[photo].tags.indexOf(tag) < 0) {
        tagCounts.value[tag] += 1;
      }
      if (tags.value.indexOf(tag) >= 0) {
        tags.value.splice(tags.value.indexOf(tag), 1);
      }
      tags.value.splice(0, 0, tag);
    });
    files.value[photo].tags.forEach((tag) => {
      if (t.indexOf(tag) < 0) {
        tagCounts.value[tag] -= 1;
        if (tagCounts.value[tag] <= 0) {
          delete tagCounts.value[tag];
          tags.value.splice(tags.value.indexOf(tag), 1);
        }
      }
    });
    files.value[photo].tags = t;
    await database?.insert(files.value[photo]);
  }

  /**
   * Sets a photo's title.
   * @param photo - The photo.
   * @param title - The title to set.
   */
  async function setTitle(photo: string, title: string) {
    files.value[photo].data.title = title;
    await database?.insert(files.value[photo]);
  }

  /**
   * Sets a photo's description.
   * @param photo - The photo.
   * @param description - The description to set.
   */
  async function setDescription(photo: string, description: string) {
    files.value[photo].data.description = description;
    await database?.insert(files.value[photo]);
  }

  /**
   * Sets a photo's locationApprox.
   * @param photo - The photo.
   * @param locationApprox - The value to set.
   */
  async function setLocationApprox(photo: string, locationApprox: boolean) {
    files.value[photo].data.locationApprox = locationApprox;
    await database?.insert(files.value[photo]);
  }

  /**
   * Loads photos from the database.
   */
  async function loadPhotos() {
    if (database) {
      files.value = {};
      (await database.selectAll(Photo)).forEach((photo) => {
        files.value[photo.data.name] = photo;
        photo.tags.forEach((tag) => {
          if (tags.value.indexOf(tag) < 0) {
            tags.value.push(tag);
          }
        });
      });
      groups.value = [];
      (await database.selectAll(Group)).forEach((group) => {
        groups.value.push(group);
      });
    }
    return files.value;
  }

  /**
   * Removes database entries for deleted photos.
   * @param photo - The name of the photo to remove.
   */
  async function removeDeleted(photo: string) {
    await database?.execute(`DELETE FROM Photo WHERE Name='${photo}'`);
    delete files.value[photo];
  }

  function setFiles(data: Record<string, Photo>) {
    files.value = data;
  }

  return {
    files,
    groups,
    workingDir,
    tags,
    tagCounts,
    locations,
    addFile,
    setWorkingDir,
    setPhotoData,
    setThumbnail,
    photoCount,
    setLocation,
    moveTagsToFront,
    setVideo,
    addGroup,
    groupNames,
    setRating,
    setDuplicate,
    getByGroup,
    setGroup,
    removeGroup,
    updateTags,
    setTitle,
    setDescription,
    setLocationApprox,
    loadPhotos,
    removeDeleted,
    setFiles,
  };
});
