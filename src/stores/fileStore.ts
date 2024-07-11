import { EventEmitter } from 'ee-ts';
import { Group } from '../classes/Group';
import { Photo } from '../classes/Photo';
import { TauriDatabase } from '@/classes/TauriDatabase';
import { Tag } from '~/classes/Tag';
import { Graph } from '~/classes/Graph';
import { GraphNode } from '~/classes/GraphNode';
import type { FileEntry } from '@tauri-apps/api/fs';
import { Place } from '~/classes/Place';
import type { PlaceType, Position } from '~/classes/Map';
import { Layer } from '~/classes/Layer';
import { Shape, type ShapeType } from '~/classes/Shape';

class FileStore extends EventEmitter<{
  updateFilters(): void;
  updatePhoto(photo: string): void;
  updateLocations(): void;
  saving(value: boolean): void;
  saveError(): void;
  thumbnailProgress(progress: number): void;
  validationUpdate(photo: string): void;
}> {
  public advTags: Tag[] = [];

  public database: TauriDatabase | null = null;

  public files: Record<string, Photo> = {};

  public readonly filters = {
    disabledTags: [],
    enabledTags: [],
    filterMode: 'AND',
    filterPos: '',
    filterDate: '',
    hideDuplicates: true,
    hideLocated: false,
    hideTagged: false,
    onlyError: false,
    onlyLocated: false,
    onlyTagged: false,
  };

  public generatingThumbnails = false;

  public groups: Group[] = [];

  public groupNames: string[] = [];

  public initialized = false;

  public photoCount = 0;

  public saveError = false;

  public tagCounts: Record<string, number> = {};

  public tags: string[] = [];

  public thumbnailProgress = 0;

  public workingDir = '';

  public places: Record<string, Place> = {};

  private newestPlace = '';

  public layers: Record<string, Layer> = {};

  public shapes: Record<string, Shape> = {};

  public calendarViewDate = new Date();

  /**
   * Sets the working dir name.
   * @param path - The path to the working dir.
   */
  public async setWorkingDir(path: string) {
    this.workingDir = path;
    const { join } = await import('@tauri-apps/api/path');
    this.database = new TauriDatabase(`sqlite:${await join(path, 'photos.db')}`);
    this.database.on('startQuery', () => {
      this.emit('saving', true);
    });
    this.database.on('endQuery', () => {
      this.emit('saving', false);
    });
    this.database.on('queryError', () => {
      this.emit('saveError');
    });
  }

  /**
   * Sets a photo's thumbnail property.
   * @param photo - The photo to set for.
   * @param thumbnail - The path to the thumbnail.
   */
  private async setThumbnail(photo: string, thumbnail: string) {
    this.files[photo].data.thumbnail = thumbnail;
    await this.database?.insert(this.files[photo]);
    this.emit('updatePhoto', photo);
  }

  /**
   * Adds a group.
   * @param name - The name of the group.
   */
  public async addGroup(name: string) {
    const g = new Group({ name });
    this.groups.push(g);
    this.groupNames.push(name);
    await this.database?.insert(g);
  }

  /**
   * Sets a photo's rating.
   * @param photo - The photo to set for.
   * @param rating - The rating to set.
   */
  public async setRating(photo: string, rating: number) {
    this.files[photo].data.rating = rating;
    await this.database?.insert(this.files[photo]);
    this.emit('updatePhoto', photo);
  }

  /**
   * Sets a photo's isDuplicate marker.
   * @param photo - The photo to set for.
   * @param isDuplicate - The duplicate marker.
   */
  public async setDuplicate(photo: string, isDuplicate: boolean) {
    this.files[photo].data.isDuplicate = isDuplicate;
    await this.database?.insert(this.files[photo]);
    this.emit('updatePhoto', photo);
  }

  /**
   * Gets all photos in a group.
   * @param group - The group to get photos from.
   */
  public getByGroup(group: string) {
    return Object.values(this.files).filter((p) => p.data.photoGroup === group);
  }

  /**
   * Sets a photo's group.
   * @param photo - The photo to set.
   * @param group - The group to set.
   */
  public async setGroup(photo: string, group?: string) {
    if (group === undefined) {
      this.files[photo].data.photoGroup = '';
      return;
    }
    this.files[photo].data.photoGroup = group;
    await this.database?.insert(this.files[photo]);
    const collectedTags: string[] = [];
    this.getByGroup(group).forEach((photo) => {
      this.files[photo.data.name].tags.forEach((tag) => {
        if (collectedTags.indexOf(tag) < 0) {
          collectedTags.push(tag);
        }
      });
    });
    await this.updateTagsForGroup(photo, collectedTags);
    this.emit('updatePhoto', photo);
  }

  /**
   * Removes a photo from its group.
   * @param photo - The photo to remove from its group.
   */
  public async removeGroup(photo: string) {
    this.files[photo].data.photoGroup = '';
    await this.database?.insert(this.files[photo]);
    this.emit('updatePhoto', photo);
  }

  /**
   * Updates tags for photo groups.
   * @param photo - The base photo.
   * @param t - The list of tags.
   */
  public async updateTagsForGroup(photo: string, t: string[]) {
    const newTags: string[] = [];
    t.forEach((tag) => {
      if (this.files[photo].group === undefined || this.files[photo].firstInGroup) {
        if (!this.tagCounts[tag]) {
          this.tagCounts[tag] = 0;
        }
        if (this.files[photo].tags.indexOf(tag) < 0) {
          this.tagCounts[tag] += 1;
        }
      }
      if (this.tags.indexOf(tag) < 0) {
        this.tags.push(tag);
        newTags.push(tag);
      }
    });
    if (this.files[photo].group === undefined || this.files[photo].firstInGroup) {
      this.files[photo].tags.forEach((tag) => {
        if (t.indexOf(tag) < 0) {
          this.tagCounts[tag] -= 1;
          if (this.tagCounts[tag] <= 0) {
            delete this.tagCounts[tag];
            this.tags.splice(this.tags.indexOf(tag), 1);
          }
        }
      });
    }
    this.files[photo].tags = t;
    await this.database?.insert(this.files[photo]);
    this.sortTags();
    this.emit('updatePhoto', photo);
    // TODO: inform of newly created tags
  }

  /**
   * Adds new tags to the master list.
   * @param t - The tags to apply.
   */
  public updateTags(t: string[]) {
    const newTags: string[] = [];
    t.forEach((tag) => {
      if (this.tags.indexOf(tag) < 0) {
        this.tags.push(tag);
        newTags.push(tag);
      }
    });
    this.sortTags();
  }

  /**
   * Sets a photo's title.
   * @param photo - The photo.
   * @param title - The title to set.
   */
  public async setTitle(photo: string, title: string) {
    this.files[photo].data.title = title;
    await this.database?.insert(this.files[photo]);
    this.emit('updatePhoto', photo);
  }

  /**
   * Sets a photo's description.
   * @param photo - The photo.
   * @param description - The description to set.
   */
  public async setDescription(photo: string, description: string) {
    this.files[photo].data.description = description;
    await this.database?.insert(this.files[photo]);
    this.emit('updatePhoto', photo);
  }

  /**
   * Sets & sorts the tag list.
   * @param tags - The unsorted tags.
   */
  private sortTags() {
    const tagGraph = new Graph();
    this.tags.forEach((tag) => {
      if (!tagGraph.get(tag)) {
        tagGraph.nodes.push(new GraphNode(tag));
      }
      const adv = this.advTags.find((t) => t.data.name === tag);
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
    });
    this.tags = tagGraph.sort();
  }

  /**
   * Loads photos from the database.
   */
  public async loadPhotos() {
    if (this.database) {
      this.files = {};
      this.advTags = await this.database.selectAll(Tag);
      const tagList: string[] = [];
      const encounteredGroups: string[] = [];
      (await this.database.selectAll(Place)).forEach((place) => {
        this.places[place.Id] = place;
        place.tags.forEach((tag) => {
          if (tagList.indexOf(tag) < 0) {
            tagList.push(tag);
          }
        });
      });
      const raws: Photo[] = [];
      (await this.database.selectAll(Photo)).forEach((photo) => {
        this.files[photo.data.name] = photo;
        if (photo.data.date === null) {
          photo.data.date = '';
        }
        let firstInGroup = false;
        if (photo.group && encounteredGroups.indexOf(photo.group) < 0) {
          this.files[photo.data.name].firstInGroup = true;
          firstInGroup = true;
          encounteredGroups.push(photo.group);
        }
        if (photo.group === undefined || firstInGroup) {
          this.photoCount += 1;
          photo.tags.forEach((tag) => {
            if (tagList.indexOf(tag) < 0) {
              tagList.push(tag);
            }
            if (!this.tagCounts[tag]) {
              this.tagCounts[tag] = 0;
            }
            this.tagCounts[tag] += 1;
          });
        }
        if (photo.hasLocation && this.places[photo.data.location]) {
          this.places[photo.data.location].count += 1;
        }
        this.validateTags(photo.data.name);
        if (photo.data.raw) {
          raws.push(photo);
        }
      });
      this.groupRaws(raws);
      this.groups = await this.database.selectAll(Group);
      this.groupNames = this.groups.map((g) => g.data.name);
      (await this.database.selectAll(Layer)).forEach((layer) => {
        this.layers[layer.Id] = layer;
      });
      (await this.database.selectAll(Shape)).forEach((shape) => {
        this.shapes[shape.Id] = shape;
      });
      this.tags = tagList;
      this.sortTags();
      this.initialized = true;
    }
    return this.files;
  }

  /**
   * Removes database entries for deleted photos.
   * @param photo - The name of the photo to remove.
   */
  public async removeDeleted(photo: string) {
    await this.database?.execute(`DELETE FROM Photo WHERE Name='${photo}'`);
    delete this.files[photo];
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

  /**
   * Initializes the files object.
   * @param data - The files data.
   */
  public setFiles(data: Record<string, Photo>) {
    this.files = data;
    this.photoCount = Object.values(data).length;
  }

  /**
   * Automatically groups raw photos that already have a JPG or PNG version.
   * @param raws - The list of raw files.
   */
  public groupRaws(raws: Photo[]) {
    raws.forEach((raw) => {
      const baseName = raw.data.name.replace('.ORF', '').replace('.NRW', '');
      if (this.files[`${baseName}.JPG`]) {
        this.files[`${baseName}.JPG`].rawFile = raw.data.thumbnail;
        this.files[raw.data.name].hidden = true;
      } else if (this.files[`${baseName}.PNG`]) {
        this.files[`${baseName}.PNG`].rawFile = raw.data.thumbnail;
        this.files[raw.data.name].hidden = true;
      }
    });
  }

  /**
   * Sets a photo's date.
   * @param photo - The target photo.
   * @param date - The date to set.
   */
  public async setDate(photo: string, date: string) {
    this.files[photo].data.date = date;
    await this.database?.insert(this.files[photo]);
    this.emit('updatePhoto', photo);
  }

  /**
   * Ensures a tag exists in the Tag table.
   * @param tag - The target tag.
   */
  private async ensureAdvTag(tag: string) {
    const t = this.advTags.find((x) => x.data.name === tag);
    if (t) {
      return t;
    }
    const advTag = new Tag({ name: tag, color: '', prereqs: '', coreqs: '', incompatible: '' });
    this.advTags.push(advTag);
    await this.database?.insert(advTag);
    return advTag;
  }

  /**
   * Sets a filter's value.
   * @param key - The filter key to set.
   * @param value - The value to set.
   */
  public setFilter(key: keyof typeof this.filters, value: any) {
    console.log(`Setting filter ${key} to ${value}`);
    this.filters[key] = value;
    this.emit('updateFilters');
  }

  /**
   * Sets a tag's color.
   * @param tag - The target tag.
   * @param color - The color to set.
   */
  public async setTagColor(tag: string, color: string) {
    const t = await this.ensureAdvTag(tag);
    t.data.color = color;
    await this.database?.insert(t);
  }

  /**
   * Validates photos when a tag's requirements change.
   * @param tag - The tag that changed.
   */
  public handleTagChange(tag: string) {
    Object.entries(this.files).forEach(([name, photo]) => {
      if (photo.tags.indexOf(tag) >= 0) {
        this.validateTags(name);
      }
    });
  }

  /**
   * Sets a tag's prerequisites.
   * @param tag - The target tag.
   * @param prereqs - The prereq list.
   */
  public async setTagPrereqs(tag: string, prereqs: string[]) {
    const t = await this.ensureAdvTag(tag);
    t.prereqs = prereqs;
    await this.database?.insert(t);
  }

  /**
   * Sets a tag's prerequisites.
   * @param tag - The target tag.
   * @param coreqs - The prereq list.
   */
  public async setTagCoreqs(tag: string, coreqs: string[]) {
    const t = await this.ensureAdvTag(tag);
    t.coreqs = coreqs;
    await this.database?.insert(t);
  }

  /**
   * Sets a tag's incompatible.
   * @param tag - The target tag.
   * @param incompatible - The incompatible list.
   */
  public async setTagIncompatible(tag: string, incompatible: string[]) {
    const t = await this.ensureAdvTag(tag);
    t.incompatible = incompatible;
    await this.database?.insert(t);
  }

  /**
   * Helper method for getting a tag's color;
   * @param tag - The tag to get.
   */
  public getTagColor(tag: string) {
    const at = this.advTags.find((t) => t.data.name === tag);
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
  public validateTags(photo: string) {
    let valid = true;
    let msg = '';
    const tags = this.files[photo].tags;
    tags.forEach((tag) => {
      const a = this.advTags.find((t) => t.data.name === tag);
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
    this.files[photo].valid = valid;
    this.files[photo].validationMsg = msg;
    this.emit('updatePhoto', photo);
    this.emit('validationUpdate', photo);
  }

  /**
   * A list of photos, with the filter options applied.
   * @param filterByLocation
   * @param filterByDate
   */
  public filteredPhotos(filterByLocation = false, filterByDate = false) {
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
      filterPos,
      filterDate,
    } = this.filters;
    Object.values(this.files)
      .filter((file) => !file.hidden)
      .forEach((file) => {
        let satisfiesTags = filterMode === 'AND' || enabledTags.length === 0;
        if (
          (hideTagged && file.tags.length > 0) ||
          (onlyTagged && file.tags.length === 0) ||
          (hideLocated && file.hasLocation) ||
          (onlyLocated && !file.hasLocation) ||
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
        if (satisfiesTags && filterByLocation) {
          if (file.hasLocation && this.places[file.data.location]) {
            if (file.data.location !== filterPos) {
              satisfiesTags = false;
            }
          } else {
            satisfiesTags = false;
          }
        }
        if (satisfiesTags && filterByDate) {
          const d1 = new Date(Date.parse(filterDate));
          if (file.data.date.length > 0) {
            if (
              d1.getFullYear() !== file.date.getFullYear() ||
              d1.getMonth() !== file.date.getMonth() ||
              d1.getDate() !== file.date.getDate()
            ) {
              satisfiesTags = false;
            }
          } else {
            satisfiesTags = false;
          }
        }
        if (satisfiesTags) {
          filtered.push(file);
        }
      });
    return filtered;
  }

  /**
   * Generates thumbnails in the background.
   * @param raws - RAW photo files to generate thumbnails for.
   * @param videos - Video files to generate thumbnails for.
   */
  public async generateThumbnails(raws: FileEntry[], videos: FileEntry[]) {
    const { readDir, exists, createDir } = await import('@tauri-apps/api/fs');
    const { join, appDataDir } = await import('@tauri-apps/api/path');
    const { convertFileSrc } = await import('@tauri-apps/api/tauri');
    const { Command } = await import('@tauri-apps/api/shell');
    this.generatingThumbnails = true;
    this.thumbnailProgress = 0;
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
    };
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
      this.workingDir.replace(/[/\\]/g, '-').replace(':', ''),
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
      if (this.files[raw.path].data.thumbnail.length === 0) {
        await this.setThumbnail(raw.path, convertFileSrc(thumbnailPath));
      }
      this.files[raw.path].awaitingThumbnail = false;
      progress += 1;
      const p = Math.round((progress / total) * 100);
      if (p > lastProgressInt) {
        this.thumbnailProgress = p;
        lastProgressInt = p;
        this.emit('thumbnailProgress', this.thumbnailProgress);
      }
      this.emit('updatePhoto', raw.path);
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
      if (this.files[video.path].data.thumbnail.length === 0) {
        await this.setThumbnail(video.path, convertFileSrc(thumbnailPath));
      }
      this.files[video.path].awaitingThumbnail = false;
      progress += 1;
      const p = Math.round((progress / total) * 100);
      if (p > lastProgressInt) {
        this.thumbnailProgress = p;
        lastProgressInt = p;
        this.emit('thumbnailProgress', this.thumbnailProgress);
      }
      this.emit('updatePhoto', video.path);
    }
    this.generatingThumbnails = false;
  }

  /**
   * Gets a file.
   * @param name - The name of the file.
   * @returns The file object.
   */
  public getFile(name: string) {
    return this.files[name];
  }

  /**
   * Sets a photo's location.
   * @param photo - The target photo.
   * @param location - The location to set.
   */
  public async setLocation(photo: string, location: string) {
    if (this.files[photo].hasLocation && this.places[this.files[photo].data.location]) {
      this.places[this.files[photo].data.location].count -= 1;
    }
    this.files[photo].data.location = location;
    this.places[location].count += 1;
    await this.database?.insert(this.files[photo]);
    this.emit('updatePhoto', photo);
    this.emit('updateLocations');
  }

  /**
   * Creates a Place entry.
   * @param name - The name of the place.
   * @param pos - The latitude & longitude of the place.
   * @param layer - The layer the place belongs to.
   */
  public async createPlace(name: string, pos: Position, category: PlaceType, layer: string) {
    const p = new Place({
      name,
      lat: pos.lat,
      lng: pos.lng,
      layer,
      category,
      shape: '',
      tags: '',
      notes: '',
    });
    p.isNewestPlace = true;
    if (this.places[this.newestPlace]) {
      this.places[this.newestPlace].isNewestPlace = false;
    }
    this.newestPlace = p.Id;
    this.places[p.Id] = p;
    await this.database?.insert(p);
    return p;
  }

  /**
   * Creates a Layer entry.
   * @param name - The name of the layer.
   */
  public async createLayer(name: string) {
    const l = new Layer({ name, color: '#ff0000' });
    this.layers[l.Id] = l;
    await this.database?.insert(l);
    return l;
  }

  /**
   * Gets a list of places in the given layer.
   * @param layer - The target layer.
   */
  public getPlacesByLayer(layer: string) {
    return Object.values(this.places).filter((p) => p.data.layer === layer);
  }

  /**
   * Sets & saves a layer's color.
   * @param layer - The target layer.
   * @param color - The color to set.
   */
  public async setLayerColor(layer: string, color: string) {
    this.layers[layer].data.color = color;
    await this.database?.update(this.layers[layer]);
  }

  /**
   * Creates a shape.
   * @param type - The shape type.
   * @param points - The shape path.
   * @param layer - The layer the shape belongs to.
   * @param name - The name of the shape.
   */
  public async createShape(type: ShapeType, points: Position[], layer: string, name: string) {
    const s = new Shape({
      type,
      points: JSON.stringify(points),
      layer,
      name,
    });
    this.shapes[s.Id] = s;
    await this.database?.insert(s);
    return s;
  }

  /**
   * Links a place to a polygon.
   * @param place - The target place.
   * @param shape - The associated shape.
   */
  public async setPlaceShape(place: string, shape: Shape) {
    this.places[place].data.shape = shape.Id;
    await this.database?.update(this.places[place]);
  }

  /**
   * Updates a shape's path.
   * @param shape - The target shape.
   * @param path - The new path.
   */
  public async setShapePath(shape: string, path: Position[]) {
    this.shapes[shape].points = path;
    await this.database?.update(this.shapes[shape]);
  }

  /**
   * Sets a shape's name.
   * @param shape - The target shape.
   * @param name - The new name.
   */
  public async setShapeName(shape: string, name: string) {
    this.shapes[shape].data.name = name;
    await this.database?.update(this.shapes[shape]);
  }

  /**
   * Sets a place's name.
   * @param place - The target place.
   * @param name - The new name.
   */
  public async setPlaceName(place: string, name: string) {
    this.places[place].data.name = name;
    await this.database?.update(this.places[place]);
  }

  /**
   * Deletes a place.
   * @param place - The target place.
   */
  public async deletePlace(place: string) {
    await this.database?.delete(this.places[place]);
    delete this.places[place];
  }

  /**
   * Deletes a shape.
   * @param shape - The target shape.
   */
  public async deleteShape(shape: string) {
    await this.database?.delete(this.shapes[shape]);
    delete this.shapes[shape];
  }

  /**
   * Sets a place's layer.
   * @param place - The target place.
   * @param layer - The target layer.
   */
  public async setPlaceLayer(place: string, layer: string) {
    this.places[place].data.layer = layer;
    await this.database?.update(this.places[place]);
  }

  /**
   * Sets a shape's layer.
   * @param shape - The target shape.
   * @param layer - The target layer.
   */
  public async setShapeLayer(shape: string, layer: string) {
    this.shapes[shape].data.layer = layer;
    await this.database?.update(this.shapes[shape]);
  }

  /**
   * Sets a place's tags.
   * @param place - The target place.
   * @param tags - The tags to set.
   */
  public async setPlaceTags(place: string, tags: string[]) {
    this.places[place].tags = tags;
    await this.database?.update(this.places[place]);
  }

  /**
   * Sets a place's notes.
   * @param place - The target place.
   * @param notes - The notes to set.
   */
  public async setPlaceNotes(place: string, notes: string) {
    this.places[place].data.notes = notes;
    await this.database?.update(this.places[place]);
  }

  /**
   * Sets a place's category.
   * @param place - The target place.
   * @param category - The category to set.
   */
  public async setPlaceCategory(place: string, category: PlaceType) {
    this.places[place].data.category = category;
    await this.database?.update(this.places[place]);
  }

  /**
   * Sets a place's position.
   * @param place - The target place.
   * @param position - The position to set.
   */
  public async setPlacePosition(place: string, position: Position) {
    this.places[place].data.lat = position.lat;
    this.places[place].data.lng = position.lng;
    await this.database?.update(this.places[place]);
  }

  /**
   * Update the calendar's focused date.
   * @param date - The date to focus on.
   */
  public setCalendarViewDate(date: Date) {
    this.calendarViewDate = date;
  }
}

const f = new FileStore();
Object.getOwnPropertyNames(Object.getPrototypeOf(f)).forEach((key) => {
  if (key !== 'constructor') {
    if (typeof Object.getPrototypeOf(f)[key] === 'function') {
      f[key] = (...args: any[]) => {
        console.log(key, args);
        return Object.getPrototypeOf(f)[key].call(f, ...args);
      };
    } else {
      f[key] = Object.getPrototypeOf(f)[key].bind(f);
    }
  }
});
export const fileStore = f;
