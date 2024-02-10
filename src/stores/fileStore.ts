import { defineStore } from 'pinia';
import { computed, ref } from 'vue';
import { locToString } from '../classes/Map';
import { Group } from '../classes/Group';
import { createPhoto, Photo } from '../classes/Photo';
import { TauriDatabase } from '@/classes/TauriDatabase';
import { Tag } from '~/classes/Tag';
import { Graph } from '~/classes/Graph';
import { GraphNode } from '~/classes/GraphNode';
import type { FileEntry } from '@tauri-apps/api/fs';

export const useFileStore = defineStore('files', () => {
  let database: TauriDatabase | null = null;

  const initialized = ref(false);

  const saving = ref(false);

  const saveError = ref(false);

  const files = ref<Record<string, Photo>>({});

  const groups = ref<Group[]>([]);

  const workingDir = ref('');

  const tags = ref<string[]>([]);

  const advTags = ref<Tag[]>([]);

  const tagCounts = ref<Record<string, number>>({});

  const generatingThumbnails = ref(false);

  const thumbnailProgress = ref(0);

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
    database.on('startQuery', () => {
      saving.value = true;
    });
    database.on('endQuery', () => {
      saving.value = false;
    });
    database.on('queryError', () => {
      saveError.value = true;
    });
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
  async function setThumbnail(photo: string, thumbnail: string) {
    files.value[photo].data.thumbnail = thumbnail;
    await database?.insert(files.value[photo]);
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
    const newTags: string[] = [];
    t.forEach((tag) => {
      if (files.value[photo].group === undefined || files.value[photo].firstInGroup) {
        if (!tagCounts.value[tag]) {
          tagCounts.value[tag] = 0;
        }
        if (files.value[photo].tags.indexOf(tag) < 0) {
          tagCounts.value[tag] += 1;
        }
      }
      if (tags.value.indexOf(tag) < 0) {
        tags.value.push(tag);
        newTags.push(tag);
      }
    });
    if (files.value[photo].group === undefined || files.value[photo].firstInGroup) {
      files.value[photo].tags.forEach((tag) => {
        if (t.indexOf(tag) < 0) {
          tagCounts.value[tag] -= 1;
          if (tagCounts.value[tag] <= 0) {
            delete tagCounts.value[tag];
            tags.value.splice(tags.value.indexOf(tag), 1);
          }
        }
      });
    }
    files.value[photo].tags = t;
    await database?.insert(files.value[photo]);
    sortTags();
    // TODO: inform of newly created tags
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
   * Sets & sorts the tag list.
   * @param tags - The unsorted tags.
   */
  function sortTags() {
    const tagGraph = new Graph();
    tags.value.forEach((tag) => {
      console.log(tag);
      if (!tagGraph.get(tag)) {
        tagGraph.nodes.push(new GraphNode(tag));
      }
      const adv = advTags.value.find((t) => t.data.name === tag);
      if (adv && adv.prereqs.length > 0) {
        adv.prereqs.forEach((p) => {
          if (!tagGraph.get(p)) {
            const gn = new GraphNode(p);
            gn.links.push(tag);
            tagGraph.nodes.push(gn);
          } else {
            const gn = tagGraph.get(p) as GraphNode;
            if (gn.links.indexOf(tag) < 0) {
              tagGraph.get(p)?.links.push(tag);
            }
          }
        });
      }
      console.log(tag);
    });
    console.log(tagGraph);
    tags.value = tagGraph.sort();
  }

  /**
   * Loads photos from the database.
   */
  async function loadPhotos() {
    if (database) {
      files.value = {};
      advTags.value = await database.selectAll(Tag);
      const tagList: string[] = [];
      const encounteredGroups: string[] = [];
      (await database.selectAll(Photo)).forEach((photo) => {
        files.value[photo.data.name] = photo;
        let firstInGroup = false;
        if (photo.group && encounteredGroups.indexOf(photo.group) < 0) {
          files.value[photo.data.name].firstInGroup = true;
          firstInGroup = true;
          encounteredGroups.push(photo.group);
        }
        if (photo.group === undefined || firstInGroup) {
          photo.tags.forEach((tag) => {
            if (tagList.indexOf(tag) < 0) {
              tagList.push(tag);
            }
            if (!tagCounts.value[tag]) {
              tagCounts.value[tag] = 0;
            }
            tagCounts.value[tag] += 1;
          });
        }
        validateTags(photo.data.name);
      });
      groups.value = await database.selectAll(Group);
      console.log(tagList);
      tags.value = tagList;
      console.log('Sorting tags');
      sortTags();
      console.log('Initialized');
      initialized.value = true;
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
    /**
     * TODO: delete thumbnails
     *       const thumbnailPath = await join(
        projectThumbnailDir,
        `${deleted.value[i].replace(/\..*$/, '')}.jpg`,
      );
      if (await exists(thumbnailPath)) {
        await removeFile(thumbnailPath);
      }
     */
  }

  function setFiles(data: Record<string, Photo>) {
    files.value = data;
  }

  /**
   * Sets a photo's date.
   * @param photo - The target photo.
   * @param date - The date to set.
   */
  async function setDate(photo: string, date: string) {
    files.value[photo].data.date = date;
    await database?.insert(files.value[photo]);
  }

  /**
   * Ensures a tag exists in the Tag table.
   * @param tag - The target tag.
   */
  async function ensureAdvTag(tag: string) {
    const t = advTags.value.find((x) => x.data.name === tag);
    if (t) {
      return t;
    }
    const advTag = new Tag({ name: tag, color: '', prereqs: '', coreqs: '', incompatible: '' });
    advTags.value.push(advTag);
    await database?.insert(advTag);
    return advTag;
  }

  /**
   * Sets a tag's color.
   * @param tag - The target tag.
   * @param color - The color to set.
   */
  async function setTagColor(tag: string, color: string) {
    const t = await ensureAdvTag(tag);
    t.data.color = color;
    await database?.insert(t);
  }

  /**
   * Validates photos when a tag's requirements change.
   * @param tag - The tag that changed.
   */
  function handleTagChange(tag: string) {
    Object.entries(files.value).forEach(([name, photo]) => {
      if (photo.tags.indexOf(tag) >= 0) {
        validateTags(name);
      }
    });
  }

  /**
   * Sets a tag's prerequisites.
   * @param tag - The target tag.
   * @param prereqs - The prereq list.
   */
  async function setTagPrereqs(tag: string, prereqs: string[]) {
    const t = await ensureAdvTag(tag);
    t.prereqs = prereqs;
    await database?.insert(t);
  }

  /**
   * Sets a tag's prerequisites.
   * @param tag - The target tag.
   * @param coreqs - The prereq list.
   */
  async function setTagCoreqs(tag: string, coreqs: string[]) {
    const t = await ensureAdvTag(tag);
    t.coreqs = coreqs;
    await database?.insert(t);
  }

  /**
   * Sets a tag's incompatible.
   * @param tag - The target tag.
   * @param incompatible - The incompatible list.
   */
  async function setTagIncompatible(tag: string, incompatible: string[]) {
    const t = await ensureAdvTag(tag);
    t.incompatible = incompatible;
    await database?.insert(t);
  }

  /**
   * Helper method for getting a tag's color;
   * @param tag - The tag to get.
   */
  function getTagColor(tag: string) {
    const at = advTags.value.find((t) => t.data.name === tag);
    if (at) {
      return at.data.color;
    }
    return 'black';
  }

  /**
   * Validates tags for a photo.
   * TODO - cache the validation status so it doesn't call this function a billion times
   * @param photo - The photo to validate.
   */
  function validateTags(photo: string) {
    let valid = true;
    let msg = '';
    const tags = files.value[photo].tags;
    tags.forEach((tag) => {
      const a = advTags.value.find((t) => t.data.name === tag);
      if (a) {
        if (a.prereqs.length > 0) {
          let allPrereqsMet = true;
          let missingPrereq = '';
          let i = 0;
          while (allPrereqsMet && i < a.prereqs.length) {
            const p = a.prereqs[i];
            allPrereqsMet = allPrereqsMet && tags.indexOf(p) >= 0;
            if (tags.indexOf(p) < 0) {
              missingPrereq = `${p} (required by ${a.data.name})`;
            }
            i += 1;
          }
          if (!allPrereqsMet) {
            valid = false;
            msg = `Missing prerequisite: ${missingPrereq}`;
          }
        }
        if (a.coreqs.length > 0) {
          let allCoreqsMet = true;
          let missingCoreq = '';
          let i = 0;
          while (allCoreqsMet && i < a.coreqs.length) {
            const c = a.coreqs[i];
            allCoreqsMet = allCoreqsMet && tags.indexOf(c) >= 0;
            if (tags.indexOf(c) < 0) {
              missingCoreq = `${c} (required by ${a.data.name})`;
            }
            i += 1;
          }
          if (!allCoreqsMet) {
            valid = false;
            msg = `Missing corequisite: ${missingCoreq}`;
          }
        }
        let i = 0;
        while (valid && i < a.incompatible.length) {
          if (tags.indexOf(a.incompatible[i]) >= 0) {
            valid = false;
            msg = `Tag '${tag}' is incompatible!`;
          }
          i += 1;
        }
      }
    });
    files.value[photo].valid = valid;
    files.value[photo].validationMsg = msg;
  }

  // Global filter options
  const filters = ref<{
    disabledTags: string[];
    enabledTags: string[];
    filterMode: 'AND' | 'OR';
    hideDuplicates: boolean;
    hideLocated: boolean;
    hideTagged: boolean;
    onlyError: boolean;
    onlyLocated: boolean;
    onlyTagged: boolean;
  }>({
    disabledTags: [],
    enabledTags: [],
    filterMode: 'AND',
    hideDuplicates: true,
    hideLocated: false,
    hideTagged: false,
    onlyError: false,
    onlyLocated: false,
    onlyTagged: false,
  });

  // A list of photos, with the filter options applied
  const filteredPhotos = computed(() => {
    const filtered: Photo[] = [];
    const {
      filterMode,
      disabledTags,
      enabledTags,
      hideDuplicates,
      hideLocated,
      hideTagged,
      onlyError,
      onlyLocated,
      onlyTagged,
    } = filters.value;
    Object.values(files.value).forEach((file) => {
      let satisfiesTags = filterMode === 'AND' || enabledTags.length === 0;
      if (
        (hideTagged && file.tags.length > 0) ||
        (onlyTagged && file.tags.length === 0) ||
        (hideLocated && file.location !== undefined) ||
        (onlyLocated && file.location === undefined) ||
        (onlyError && file.valid) ||
        (hideDuplicates && file.data.isDuplicate)
      ) {
        satisfiesTags = false;
      }
      if (satisfiesTags) {
        enabledTags.forEach((tag) => {
          if (filterMode === 'OR' && file.tags.indexOf(tag) >= 0) {
            satisfiesTags = true;
          } else if (filterMode === 'AND' && file.tags.indexOf(tag) < 0) {
            satisfiesTags = false;
          }
        });
        disabledTags.forEach((tag) => {
          if (file.tags.indexOf(tag) >= 0) {
            satisfiesTags = false;
          }
        });
      }
      if (satisfiesTags) {
        filtered.push(file);
      }
    });
    return filtered;
  });

  /**
   * Generates thumbnails in the background.
   * @param raws - RAW photo files to generate thumbnails for.
   * @param videos - Video files to generate thumbnails for.
   */
  async function generateThumbnails(raws: FileEntry[], videos: FileEntry[]) {
    const { readDir, exists, createDir, removeFile } = await import('@tauri-apps/api/fs');
    const { join, appDataDir } = await import('@tauri-apps/api/path');
    const { convertFileSrc } = await import('@tauri-apps/api/tauri');
    const { Command } = await import('@tauri-apps/api/shell');
    generatingThumbnails.value = true;
    thumbnailProgress.value = 0;
    const total = raws.length + videos.length;
    let progress = 0;
    let lastProgressInt = 0;
    /**
     * Helper function to clean a thumbnail file name.
     * @param path - The path to the thumbnail file. 
     * @returns The "cleaned" thumbnail name.
     */
    const clean = (path: string) => {
      return path.replace(/[/\\]/g, '-').replace(':', '');
    }
    const dir = await appDataDir();
    if (!(await exists(dir))) {
      await createDir(dir);
    }
    const thumbnailDir = await join(dir, 'thumbnails');
    if (!(await exists(thumbnailDir))) {
      await createDir(thumbnailDir);
    }
    const projectThumbnailDir = await join(
      thumbnailDir,
      workingDir.value.replace(/[/\\]/g, '-').replace(':', ''),
    );
    if (!(await exists(projectThumbnailDir))) {
      await createDir(projectThumbnailDir);
    }
    const thumbnails = (await readDir(projectThumbnailDir)).map((p) => p.name);
    for (const raw of raws) {
      const thumbnailFile = `${clean(raw.path as string).replace(/\..*$/, '')}.jpg`;
      const thumbnailPath = `${projectThumbnailDir}/${thumbnailFile}`; // tauri's join() slowed down this one line by like 10,000%
      if (thumbnails.indexOf(thumbnailFile) < 0) {
        const convertOutput = await new Command('magick', [raw.path, thumbnailPath]).execute();
        if (convertOutput.code !== 0) {
          console.error(convertOutput.stderr);
        }
        const resizeOutput = await new Command('magick', [
          thumbnailPath,
          '-resize',
          '800x800',
          thumbnailPath,
        ]).execute();
        if (resizeOutput.code !== 0) {
          console.error(resizeOutput.stderr);
        }
      }
      if (files.value[raw.path].data.thumbnail.length === 0) {
        await setThumbnail(raw.path, convertFileSrc(thumbnailPath));
      }
      files.value[raw.path].awaitingThumbnail = false;
      progress += 1;
      const p = Math.round((progress / total) * 100);
      if (p > lastProgressInt) {
        thumbnailProgress.value = p;
        lastProgressInt = p;
      }
    }
    for (const video of videos) {
      const thumbnailFile = `${clean(video.path as string).replace(/\..*$/, '')}.png`;
      const thumbnailPath = `${projectThumbnailDir}/${thumbnailFile}`;
      if (thumbnails.indexOf(thumbnailFile) < 0) {
        const convertOutput = await new Command('ffmpeg', [
          '-i',
          video.path,
          '-ss',
          '00:00:01.00',
          '-vframes',
          '1',
          thumbnailPath,
        ]).execute();
        if (convertOutput.code !== 0) {
          console.error(convertOutput.stderr);
        }
      }
      if (files.value[video.path].data.thumbnail.length === 0) {
        await setThumbnail(video.path, convertFileSrc(thumbnailPath));
      }
      files.value[video.path].awaitingThumbnail = false;
      progress += 1;
      const p = Math.round((progress / total) * 100);
      if (p > lastProgressInt) {
        thumbnailProgress.value = p;
        lastProgressInt = p;
      }
    }
    generatingThumbnails.value = false;
  }

  return {
    saving,
    saveError,
    files,
    groups,
    workingDir,
    tags,
    advTags,
    tagCounts,
    locations,
    generatingThumbnails,
    thumbnailProgress,
    addFile,
    setWorkingDir,
    setPhotoData,
    setThumbnail,
    photoCount,
    setLocation,
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
    setDate,
    initialized,
    setTagColor,
    handleTagChange,
    setTagPrereqs,
    setTagCoreqs,
    setTagIncompatible,
    getTagColor,
    validateTags,
    filters,
    filteredPhotos,
    generateThumbnails,
  };
});
