import { FileEntry } from '@tauri-apps/api/fs';
import { defineStore } from 'pinia';
import { computed, ref } from 'vue';
import { locToString } from '../classes/Map';
import { createPhoto, Photo } from '../classes/Photo';

export const useFileStore = defineStore('files', () => {
  const files = ref<Record<string, Photo>>({});

  const groups = ref<Record<string, string[]>>({});

  const workingDir = ref('');

  const tags = ref<string[]>([]);

  const tagCounts = ref<Record<string, number>>({});

  const locations = computed(() => {
    const locRecord: Record<string, number> = {};
    Object.values(files.value).forEach((file) => {
      if (file.location !== undefined) {
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
  function addFile(file: FileEntry) {
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
  function setWorkingDir(path: string) {
    workingDir.value = path;
  }

  /**
   * Sets the stored photo data for a file.
   * @param name - The name of the file to set.
   * @param data - The data to set.
   */
  function setPhotoData(name: string, data: Photo) {
    files.value[name] = data;
    if (data.group) {
      setGroup(name, data.group);
    }
    updateTags(name, data.tags);
    data.tags.forEach((tag) => {
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
    files.value[photo].thumbnail = thumbnail;
  }

  const photoCount = computed(() => {
    return Object.values(files.value).length;
  });

  /**
   * Sets the photo's location data.
   * @param photo - The target photo.
   * @param location - The location.
   */
  function setLocation(photo: string, location: { lat: number, lng: number }) {
    files.value[photo].location = location;
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
  function setVideo(photo: string) {
    files.value[photo].video = true;
  }

  /**
   * Adds a group.
   * @param name - The name of the group.
   * @param items - Items to initialize the group with.
   */
  function addGroup(name: string, items: string[]) {
    groups.value[name] = items;
  }

  /**
   * Gets a list of group names.
   */
  const groupNames = computed(() => {
    return Object.keys(groups.value);
  });

  /**
   * Sets a photo's rating.
   * @param photo - The photo to set for.
   * @param rating - The rating to set.
   */
  function setRating(photo: string, rating: number) {
    files.value[photo].rating = rating;
  }

  /**
   * Sets a photo's isDuplicate marker.
   * @param photo - The photo to set for.
   * @param isDuplicate - The duplicate marker.
   */
  function setDuplicate(photo: string, isDuplicate: boolean) {
    files.value[photo].isDuplicate = isDuplicate;
  }

  /**
   * Sets a photo's group.
   * @param photo - The photo to set.
   * @param group - The group to set.
   */
  function setGroup(photo: string, group: string) {
    const fgroup = files.value[photo].group;
    if (fgroup && groups.value[fgroup]) {
      // Remove from previous group
      groups.value[fgroup].splice(groups.value[fgroup].indexOf(photo), 1);
    }
    files.value[photo].group = group;
    if (!groups.value[group]) {
      groups.value[group] = [];
    }
    if (groups.value[group].indexOf(photo) < 0) {
      groups.value[group].push(photo);
    }
    const collectedTags: string[] = [];
    groups.value[group].forEach((photo) => {
      files.value[photo].tags.forEach((tag) => {
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
  function removeGroup(photo: string) {
    const fgroup = files.value[photo].group;
    if (fgroup) {
      groups.value[fgroup].splice(groups.value[fgroup].indexOf(photo), 1);
    }
    delete files.value[photo].group;
  }

  /**
 * Adds new tags to the master list.
 * @param photo - The photo to apply tags to.
 * @param t - The tags to apply.
 * @param propagate - If other photos in the group should be updated.
 */
 function updateTags(photo: string, t: string[], propagate = true) {
  t.forEach((tag) => {
    if (!tagCounts.value[tag]) {
      tagCounts.value[tag] = 0;
    }
    if (files.value[photo].tags.indexOf(tag) < 0) {
      tagCounts.value[tag] += 1;
    }
    if (tags.value.indexOf(tag) < 0) {
      tags.value.push(tag);
    }
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
  const fgroup = files.value[photo].group;
  if (fgroup && propagate) {
    groups.value[fgroup].forEach((p) => {
      if (p !== photo) {
        updateTags(p, t, false);
      }
    });
  }
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
    setGroup,
    removeGroup,
    updateTags,
  };
});
