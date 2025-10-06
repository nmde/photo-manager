import { EventEmitter } from 'ee-ts';
import { Group } from '../classes/Group';
import { Photo } from '../classes/Photo';
import { TauriDatabase } from '../classes/TauriDatabase';
import { Tag } from '../classes/Tag';
import { Graph } from '../classes/Graph';
import { GraphNode } from '../classes/GraphNode';
import { Place } from '../classes/Place';
import type { PlaceType, Position } from '../classes/Map';
import { Layer } from '../classes/Layer';
import { Shape, type ShapeType } from '../classes/Shape';
import { JournalEntry } from '../classes/JournalEntry';
import { Activity } from '../classes/Activity';
import { Person } from '../classes/Person';
import { PersonCategory } from '../classes/PersonCategory';
import { Setting, type SettingKey } from '../classes/Setting';
import { Camera } from '../classes/Camera';
import { WikiPage } from '../classes/WikiPage';

export type FolderStructure = {
  dirs: string[];
  files: string[];
};

type SearchTerm = {
  type: 'rule' | 'tag';
  target?: string;
  comparison?: string;
  value: string;
  negated: boolean;
};

export const moods = [
  {
    color: '#F44336',
    label: 'Awful',
    value: 0,
  },
  {
    color: '#FF9800',
    label: 'Bad',
    value: 1,
  },
  {
    color: '#2196F3',
    label: 'Meh',
    value: 2,
  },
  {
    color: '#4CAF50',
    label: 'Good',
    value: 3,
  },
  {
    color: '#009688',
    label: 'Awesome',
    value: 4,
  },
];

function ab2b64(arrayBuffer: ArrayBuffer) {
  return btoa(String.fromCharCode.apply(null, new Uint8Array(arrayBuffer)));
}

function b642ab(base64string: string) {
  return Uint8Array.from(atob(base64string), c => c.charCodeAt(0));
}

export function formatDate(date: Date) {
  return `${date.getFullYear()}-${date.getMonth() + 1}-${date.getDate()}`;
}
class FileStore extends EventEmitter<{
  updatePhoto(photo: Photo): void;
  updateLocations(): void;
  saving(value: boolean): void;
  saveError(): void;
  thumbnailProgress(progress: number): void;
  validationUpdate(photo: string): void;
  encryptionProgress(progress: number): void;
  decrypted(): void;
  toggleTheme(): void;
  search(results: Photo[]): void;
  updateWiki(): void;
}> {
  public activities: Record<string, Activity> = {};

  public advTags: Tag[] = [];

  public database: TauriDatabase | null = null;

  public files: Record<string, Photo> = {};

  public generatingThumbnails = false;

  public groups: Group[] = [];

  public groupNames: string[] = [];

  public initialized = false;

  public journals: Record<string, JournalEntry> = {};

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

  public peopleCategories: Record<string, PersonCategory> = {};

  public peopleMap: Record<string, Person[]> = {};

  public people: Record<string, Person> = {};

  public dateMap: Record<string, Photo[]> = {};

  public locationMap: Record<string, Photo[]> = {};

  public peoplePhotoMap: Record<string, Photo[]> = {};

  public photographerMap: Record<string, Photo[]> = {};

  public folder: FolderStructure = {
    dirs: [],
    files: [],
  };

  public viewMode = 0;

  public sort = [0, 1];

  private settingsRecord: Record<string, Setting> = {};

  public settings: {
    [key in SettingKey]: boolean | string;
  } = {
    encrypt: false,
    theme: false,
  };

  public encrypted = false;

  private key!: CryptoKey;

  public cameras: Record<string, Camera> = {};

  public theme = false;

  public firstDate = new Date();

  public lastDate = new Date();

  public query: string[] = [];

  public wikiPages: Record<string, WikiPage> = {};

  /**
   * Sets the working dir name.
   * @param path - The path to the working dir.
   */
  public setWorkingDir = async (path: string) => {
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
    const f = this.files[photo];
    if (f) {
      f.data.thumbnail = thumbnail;
      await this.database?.insert(f);
      this.emit('updatePhoto', f);
    }
  }

  /**
   * Adds a group.
   * @param name - The name of the group.
   */
  public addGroup = async (name: string) => {
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
  public setRating = async (photo: string, rating: number) => {
    const f = this.files[photo];
    if (f) {
      f.data.rating = rating;
      await this.database?.insert(f);
      this.emit('updatePhoto', f);
    }
  }

  /**
   * Sets a photo's isDuplicate marker.
   * @param photo - The photo to set for.
   * @param isDuplicate - The duplicate marker.
   */
  public setDuplicate = async (photo: string, isDuplicate: boolean) => {
    const f = this.files[photo];
    if (f) {
      f.data.isDuplicate = isDuplicate;
      await this.database?.insert(f);
      this.emit('updatePhoto', f);
    }
  }

  /**
   * Gets all photos in a group.
   * @param group - The group to get photos from.
   */
  public getByGroup(group: string) {
    return Object.values(this.files).filter(p => p.data.photoGroup === group);
  }

  /**
   * Sets a photo's group.
   * @param photo - The photo to set.
   * @param group - The group to set.
   */
  public setGroup = async (photo: string, group?: string) => {
    const f = this.files[photo];
    if (f) {
      if (group === undefined) {
        f.data.photoGroup = '';
        return;
      }
      f.data.photoGroup = group;
      await this.database?.insert(f);
      const collectedTags: string[] = [];
      const collectedPeople: string[] = [];
      let location = this.files[photo]?.data.location;
      let photographer = this.files[photo]?.data.photographer;
      this.getByGroup(group).forEach(photo => {
        this.files[photo.data.name]?.tags.forEach(tag => {
          if (collectedTags.indexOf(tag) < 0) {
            collectedTags.push(tag);
          }
        });
        this.files[photo.data.name]?.people.forEach(person => {
          if (collectedPeople.indexOf(person) < 0) {
            collectedPeople.push(person);
          }
        });
        if (location?.length === 0 && photo.data.location.length > 0) {
          location = photo.data.location;
        }
        if (photographer?.length === 0 && photo.data.photographer.length > 0) {
          photographer = photo.data.photographer;
        }
      });
      if (!this.files[photo]?.hasLocation && location) {
        await this.setLocation(photo, location);
      }
      if (this.files[photo]?.data.photographer.length === 0 && photographer) {
        await this.setPhotographer(photo, photographer);
      }
      await this.updatePeopleForGroup(photo, collectedPeople);
      await this.updateTagsForGroup(photo, collectedTags);
      this.emit('updatePhoto', f);
    }
  }

  /**
   * Removes a photo from its group.
   * @param photo - The photo to remove from its group.
   */
  public removeGroup = async (photo: string) => {
    const f = this.files[photo];
    if (f) {
      f.data.photoGroup = '';
      await this.database?.insert(f);
      this.emit('updatePhoto', f);
    }
  }

  /**
   * Updates tags for photo groups.
   * @param photo - The base photo.
   * @param t - The list of tags.
   */
  public updateTagsForGroup = async (photo: string, t: string[]) => {
    const newTags: string[] = [];
    t.forEach(tag => {
      if (this.files[photo]?.group === undefined || this.files[photo].firstInGroup) {
        if (!this.tagCounts[tag]) {
          this.tagCounts[tag] = 0;
        }
        if (this.files[photo] && this.files[photo].tags.indexOf(tag) < 0) {
          this.tagCounts[tag] += 1;
        }
      }
      if (this.tags.indexOf(tag) < 0) {
        this.tags.push(tag);
        newTags.push(tag);
      }
    });
    if (this.files[photo]?.group === undefined || this.files[photo]?.firstInGroup) {
      this.files[photo]?.tags.forEach(tag => {
        if (t.indexOf(tag) < 0 && this.tagCounts[tag]) {
          this.tagCounts[tag] -= 1;
          if (this.tagCounts[tag] <= 0) {
            delete this.tagCounts[tag];
            this.tags.splice(this.tags.indexOf(tag), 1);
          }
        }
      });
    }
    const f = this.files[photo];
    if (f) {
      f.tags = t;
      await this.database?.insert(f);
      this.sortTags();
      this.emit('updatePhoto', f);
    }
    // TODO: inform of newly created tags
  }

  /**
   * Updates people for photo groups.
   * @param photo - The base photo.
   * @param p - The list of people.
   */
  public async updatePeopleForGroup(photo: string, p: string[]) {
    const f = this.files[photo];
    if (f) {
      p.forEach(person => {
        if (f.group === undefined || f.firstInGroup) {
          if (f.people.indexOf(person) < 0 && this.people[person]) {
            this.people[person].count += 1;
          }
        }
      });
      if (f.group === undefined || f.firstInGroup) {
        f.people.forEach(person => {
          if (p.indexOf(person) < 0 && this.people[person]) {
            this.people[person].count -= 1;
          }
        });
      }
      f.people = p;
      await this.database?.insert(f);
      this.emit('updatePhoto', f);
    }
  }

  /**
   * Adds new tags to the master list.
   * @param t - The tags to apply.
   */
  public updateTags = (t: string[]) => {
    const newTags: string[] = [];
    t.forEach(tag => {
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
  public setTitle = async (photo: string, title: string) => {
    const file = this.files[photo];
    if (file) {
      file.data.title = title;
      await this.database?.insert(file);
      this.emit('updatePhoto', file);
    }
  }

  /**
   * Sets a photo's description.
   * @param photo - The photo.
   * @param description - The description to set.
   */
  public setDescription = async (photo: string, description: string) => {
    const file = this.files[photo];
    if (file) {
      file.data.description = description;
      await this.database?.insert(file);
      this.emit('updatePhoto', file);
    }
  }

  /**
   * Sets & sorts the tag list.
   * @param tags - The unsorted tags.
   */
  private sortTags() {
    const tagGraph = new Graph();
    this.tags.forEach(tag => {
      if (!tagGraph.get(tag)) {
        tagGraph.nodes.push(new GraphNode(tag));
      }
      const adv = this.advTags.find(t => t.data.name === tag);
      if (adv && adv.prereqs.length > 0) {
        adv.prereqs.forEach(p => {
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

  private normalizeJournalDate(date: string) {
    return formatDate(new Date(date));
  }

  /**
   * Loads photos from the database.
   */
  public loadPhotos = async () => {
    if (this.database) {
      this.files = {};
      this.advTags = await this.database.selectAll(Tag);
      const tagList: string[] = [];
      const encounteredGroups: string[] = [];
      (await this.database.selectAll(Place)).forEach(place => {
        this.places[place.Id] = place;
        this.locationMap[place.Id] = [];
        place.tags.forEach(tag => {
          if (tagList.indexOf(tag) < 0) {
            tagList.push(tag);
          }
        });
      });
      (await this.database.selectAll(PersonCategory)).forEach(pcat => {
        this.peopleCategories[pcat.Id] = pcat;
        this.peopleMap[pcat.Id] = [];
      });
      (await this.database.selectAll(Person)).forEach(person => {
        this.peopleMap[person.data.category]?.push(person);
        this.people[person.Id] = person;
      });
      (await this.database.selectAll(Camera)).forEach(camera => {
        this.cameras[camera.Id] = camera;
      });
      const raws: Photo[] = [];
      (await this.database.selectAll(Photo)).forEach(photo => {
        this.files[photo.data.name] = photo;
        if (photo.data.date === null) {
          photo.data.date = '';
        }
        let firstInGroup = false;
        const f = this.files[photo.data.name];
        if (photo.group && encounteredGroups.indexOf(photo.group) < 0 && f) {
          f.firstInGroup = true;
          firstInGroup = true;
          encounteredGroups.push(photo.group);
        }
        if (photo.group === undefined || firstInGroup) {
          this.photoCount += 1;
          photo.tags.forEach(tag => {
            if (tagList.indexOf(tag) < 0) {
              tagList.push(tag);
            }
            if (!this.tagCounts[tag]) {
              this.tagCounts[tag] = 0;
            }
            this.tagCounts[tag] += 1;
          });
        }
        const p = this.places[photo.data.location];
        if (photo.hasLocation && p) {
          p.count += 1;
          if (!this.locationMap[photo.data.location]) {
            this.locationMap[photo.data.location] = [];
          }
          this.locationMap[photo.data.location]?.push(photo);
        }
        photo.people.forEach(id => {
          if (this.people[id]) {
            this.people[id].count += 1;
          }
          if (!this.peoplePhotoMap[id]) {
            this.peoplePhotoMap[id] = [];
          }
          this.peoplePhotoMap[id].push(photo);
        });
        const p2 = this.people[photo.data.photographer];
        if (photo.data.photographer !== undefined && p2) {
          p2.photographerCount += 1;
          if (!this.photographerMap[photo.data.photographer]) {
            this.photographerMap[photo.data.photographer] = [];
          }
          this.photographerMap[photo.data.photographer]?.push(photo);
        }
        if (photo.hasDate) {
          const date = formatDate(photo.date);
          if (!this.dateMap[date]) {
            this.dateMap[date] = [];
          }
          this.dateMap[date].push(photo);
          if (photo.date < this.firstDate) {
            this.firstDate = photo.date;
          }
          if (photo.date > this.lastDate) {
            this.lastDate = photo.date;
          }
        }
        const c = this.cameras[photo.data.camera];
        if (photo.data.camera && photo.data.camera.length > 0 && c) {
          c.count += 1;
        }
        this.validateTags(photo.data.name);
        if (photo.data.raw) {
          raws.push(photo);
        }
      });
      this.groupRaws(raws);
      this.groups = await this.database.selectAll(Group);
      this.groupNames = this.groups.map(g => g.data.name);
      (await this.database.selectAll(Layer)).forEach(layer => {
        this.layers[layer.Id] = layer;
      });
      (await this.database.selectAll(Shape)).forEach(shape => {
        this.shapes[shape.Id] = shape;
      });
      (await this.database.selectAll(Activity)).forEach(activity => {
        this.activities[activity.Id] = activity;
      });
      (await this.database.selectAll(Setting)).forEach(setting => {
        this.settings[setting.data.setting] = setting.data.value;
        this.settingsRecord[setting.data.setting] = setting;
        if (setting.data.setting === 'encrypt' && typeof setting.data.value === 'boolean') {
          this.encrypted = setting.data.value;
        } else if (setting.data.setting === 'theme' && typeof setting.data.value === 'boolean') {
          this.theme = setting.data.value;
          if (this.theme) {
            this.emit('toggleTheme');
          }
        }
      });
      (await this.database.selectAll(JournalEntry)).forEach(entry => {
        const d = this.normalizeJournalDate(entry.data.date);
        this.journals[d] = entry;
        if (entry.data.activities.length > 0) {
          this.journals[d].activities = entry.data.activities
            .split(',')
            .map(a => this.activities[a])
            .filter(a => a !== undefined);
        }
      });
      (await this.database.selectAll(WikiPage)).forEach(page => {
        this.wikiPages[page.Id] = page;
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
  public removeDeleted = async (photo: string) => {
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
  public setFiles = (data: Record<string, Photo>) => {
    this.files = data;
    this.photoCount = Object.values(data).length;
  }

  /**
   * Automatically groups raw photos that already have a JPG or PNG version.
   * @param raws - The list of raw files.
   */
  public groupRaws = (raws: Photo[]) => {
    raws.forEach(raw => {
      const baseName = raw.data.name.replace('.ORF', '').replace('.NRW', '');
      const jpg = this.files[`${baseName}.JPG`];
      const png = this.files[`${baseName}.PNG`];
      const base = this.files[raw.data.name];
      if (jpg && base) {
        jpg.rawFile = raw.data.thumbnail;
        base.hidden = true;
      } else if (png && base) {
        png.rawFile = raw.data.thumbnail;
        base.hidden = true;
      }
    });
  }

  /**
   * Sets a photo's date.
   * @param photo - The target photo.
   * @param date - The date to set.
   */
  public setDate = async (photo: string, date: string) => {
    const f = this.files[photo];
    if (f) {
      if (f.data.date.length > 0) {
        const oldDate = formatDate(f.date);
        if (this.dateMap[oldDate]) {
          const idx = this.dateMap[oldDate].findIndex(p => p.data.name === photo);
          if (idx >= 0) {
            this.dateMap[oldDate].splice(idx, 1);
          }
        }
      }
      f.data.date = date;
      const d = formatDate(f.date);
      if (!this.dateMap[d]) {
        this.dateMap[d] = [];
      }
      if (f.date < this.firstDate) {
        this.firstDate = f.date;
      }
      if (f.date > this.lastDate) {
        this.lastDate = f.date;
      }
      this.dateMap[d].push(f);
      await this.database?.insert(f);
      this.emit('updatePhoto', f);
    }
  }

  /**
   * Ensures a tag exists in the Tag table.
   * @param tag - The target tag.
   */
  private async ensureAdvTag(tag: string) {
    const t = this.advTags.find(x => x.data.name === tag);
    if (t) {
      return t;
    }
    const advTag = new Tag({ name: tag, color: '', prereqs: '', coreqs: '', incompatible: '' });
    this.advTags.push(advTag);
    await this.database?.insert(advTag);
    return advTag;
  }

  /**
   * Sets a tag's color.
   * @param tag - The target tag.
   * @param color - The color to set.
   */
  public setTagColor = async (tag: string, color: string) => {
    const t = await this.ensureAdvTag(tag);
    t.data.color = color;
    await this.database?.insert(t);
  }

  /**
   * Validates photos when a tag's requirements change.
   * @param tag - The tag that changed.
   */
  public handleTagChange = (tag: string) => {
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
  public setTagPrereqs = async (tag: string, prereqs: string[]) => {
    const t = await this.ensureAdvTag(tag);
    t.prereqs = prereqs;
    await this.database?.insert(t);
  }

  /**
   * Sets a tag's prerequisites.
   * @param tag - The target tag.
   * @param coreqs - The prereq list.
   */
  public setTagCoreqs = async (tag: string, coreqs: string[]) => {
    const t = await this.ensureAdvTag(tag);
    t.coreqs = coreqs;
    await this.database?.insert(t);
  }

  /**
   * Sets a tag's incompatible.
   * @param tag - The target tag.
   * @param incompatible - The incompatible list.
   */
  public setTagIncompatible = async (tag: string, incompatible: string[]) => {
    const t = await this.ensureAdvTag(tag);
    t.incompatible = incompatible;
    await this.database?.insert(t);
  }

  /**
   * Helper method for getting a tag's color;
   * @param tag - The tag to get.
   */
  public getTagColor = (tag: string) => {
    const at = this.advTags.find(t => t.data.name === tag);
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
  public validateTags = (photo: string) => {
    let valid = true;
    let msg = '';
    const tags = this.files[photo]?.tags;
    tags?.forEach(tag => {
      const a = this.advTags.find(t => t.data.name === tag);
      if (a) {
        if (a.prereqs.length > 0) {
          let allPrereqsMet = true;
          let missingPrereq = '';
          let i = 0;
          while (allPrereqsMet && i < a.prereqs.length) {
            const p = a.prereqs[i];
            if (p) {
              allPrereqsMet = allPrereqsMet && tags.indexOf(p) >= 0;
              if (tags.indexOf(p) < 0) {
                missingPrereq = `${p} (required by ${a.data.name})`;
              }
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
            if (c) {
              allCoreqsMet = allCoreqsMet && tags.indexOf(c) >= 0;
              if (tags.indexOf(c) < 0) {
                missingCoreq = `${c} (required by ${a.data.name})`;
              }
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
          const c = a.incompatible[i];
          if (c && tags.indexOf(c) >= 0) {
            valid = false;
            msg = `Tag '${tag}' is incompatible!`;
          }
          i += 1;
        }
      }
    });
    const f = this.files[photo];
    if (f) {
      f.valid = valid;
      f.validationMsg = msg;
      this.emit('updatePhoto', f);
      this.emit('validationUpdate', photo);
    }
  }

  /**
   * Checks a single photo to see if it matches the current search query.
   * @param photo - The photo to check.
   */
  public checkFilter = (photo: Photo) => {
    if (photo.hidden) {
      return false;
    }
    const terms = this.parseSearchTerms();
    let satisfiesRules = true;
    const rules = terms.filter(t => t.type === 'rule');
    rules.forEach(rule => {
      console.log(rule);
      if (rule.comparison === 'is') {
        if (rule.value === 'video') {
          if (rule.negated) {
            satisfiesRules = satisfiesRules && !photo.data.video;
          } else {
            satisfiesRules = satisfiesRules && photo.data.video;
          }
        }
      } else if (rule.comparison === 'of') {
        const mappedPeople = photo.people.map(p => this.people[p]?.data.name);
        if (rule.negated) {
          satisfiesRules = satisfiesRules && mappedPeople.indexOf(rule.value) < 0;
        } else {
          satisfiesRules = satisfiesRules && mappedPeople.indexOf(rule.value) >= 0;
        }
      } else if (rule.comparison === 'only') {
        const mappedPeople = photo.people.map(p => this.people[p]?.data.name);
        if (rule.negated) {
          satisfiesRules =
            satisfiesRules && (mappedPeople.indexOf(rule.value) < 0 || mappedPeople.length > 1);
        } else {
          satisfiesRules =
            satisfiesRules && mappedPeople.indexOf(rule.value) === 0 && mappedPeople.length === 1;
        }
      } else if (rule.comparison === 'by') {
        if (photo.data.photographer.length > 0) {
          const name = this.people[photo.data.photographer]?.data.name;
          if (rule.negated) {
            satisfiesRules = satisfiesRules && name !== rule.value;
          } else {
            satisfiesRules = satisfiesRules && name === rule.value;
          }
        } else {
          if (!rule.negated) {
            satisfiesRules = false;
          }
        }
      } else if (rule.comparison === 'at') {
        if (photo.hasLocation) {
          const name = this.places[photo.data.location]?.data.name;
          if (rule.negated) {
            satisfiesRules = satisfiesRules && name !== rule.value;
          } else {
            satisfiesRules = satisfiesRules && name === rule.value;
          }
        } else if (!rule.negated) {
          satisfiesRules = false;
        }
      } else if (rule.comparison === 'has') {
        if (rule.value === 'location') {
          if (rule.negated) {
            satisfiesRules = satisfiesRules && !photo.hasLocation;
          } else {
            satisfiesRules = satisfiesRules && photo.hasLocation;
          }
        } else if (rule.value === 'rating') {
          if (rule.negated) {
            satisfiesRules = satisfiesRules && !photo.hasRating;
          } else {
            satisfiesRules = satisfiesRules && photo.hasRating;
          }
        } else if (rule.value === 'photographer') {
          if (rule.negated) {
            satisfiesRules = satisfiesRules && photo.data.photographer.length === 0;
          } else {
            satisfiesRules = satisfiesRules && photo.data.photographer.length > 0;
          }
        } else if (rule.value === 'date') {
          if (rule.negated) {
            satisfiesRules = satisfiesRules && !photo.hasDate;
          } else {
            satisfiesRules = satisfiesRules && photo.hasDate;
          }
        }
      } else if (rule.comparison === 'include') {
      } else if (rule.comparison === '=') {
        if (rule.target === 'date') {
          if (rule.negated) {
            satisfiesRules = satisfiesRules && formatDate(photo.date) !== rule.value;
          } else {
            satisfiesRules = satisfiesRules && formatDate(photo.date) === rule.value;
          }
        } else if (rule.target === 'at') {
          if (photo.hasLocation) {
            if (rule.negated) {
              satisfiesRules = satisfiesRules && photo.data.location !== rule.value;
            } else {
              satisfiesRules = satisfiesRules && photo.data.location === rule.value;
            }
          } else if (!rule.negated) {
            satisfiesRules = false;
          }
        } else if (rule.target === 'of') {
          if (rule.negated) {
            satisfiesRules = satisfiesRules && photo.people.indexOf(rule.value) < 0;
          } else {
            satisfiesRules = satisfiesRules && photo.people.indexOf(rule.value) >= 0;
          }
        } else if (rule.target === 'by') {
          if (photo.data.photographer.length > 0) {
            if (rule.negated) {
              satisfiesRules = satisfiesRules && photo.data.photographer !== rule.value;
            } else {
              satisfiesRules = satisfiesRules && photo.data.photographer === rule.value;
            }
          } else {
            if (!rule.negated) {
              satisfiesRules = false;
            }
          }
        }
      } else if (rule.comparison === '<') {
        if (rule.target === 'date') {
          if (photo.hasDate) {
            const v = new Date(rule.value);
            if (rule.negated) {
              satisfiesRules = satisfiesRules && photo.date > v;
            } else {
              satisfiesRules = satisfiesRules && photo.date < v;
            }
          } else {
            satisfiesRules = false;
          }
        }
      } else if (rule.comparison === '>') {
        if (rule.target === 'date') {
          if (photo.hasDate) {
            const v = new Date(rule.value);
            if (rule.negated) {
              satisfiesRules = satisfiesRules && photo.date < v;
            } else {
              satisfiesRules = satisfiesRules && photo.date > v;
            }
          } else {
            satisfiesRules = false;
          }
        }
      }
    });
    const tags = terms.filter(t => t.type === 'tag');
    let hasAllTags = true;
    tags.forEach(tag => {
      if (tag.negated) {
        hasAllTags = hasAllTags && !photo.hasTag(tag.value);
      } else {
        hasAllTags = hasAllTags && photo.hasTag(tag.value);
      }
    });
    return satisfiesRules && hasAllTags;
  }

  /**
   * Generates thumbnails in the background.
   * @param raws - RAW photo files to generate thumbnails for.
   * @param videos - Video files to generate thumbnails for.
   */
  public generateThumbnails = async (raws: string[], videos: string[]) => {
    const { readDir, exists, mkdir } = await import('@tauri-apps/plugin-fs');
    const { join, appDataDir } = await import('@tauri-apps/api/path');
    const { convertFileSrc } = await import('@tauri-apps/api/core');
    const { Command } = await import('@tauri-apps/plugin-shell');
    this.generatingThumbnails = true;
    this.thumbnailProgress = 0;
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
      await mkdir(dir);
    }
    const thumbnailDir = await join(dir, 'thumbnails');
    if (!(await exists(thumbnailDir))) {
      await mkdir(thumbnailDir);
    }
    const projectThumbnailDir = await join(
      thumbnailDir,
      this.workingDir.replace(/[/\\]/g, '-').replace(':', ''),
    );
    if (!(await exists(projectThumbnailDir))) {
      await mkdir(projectThumbnailDir);
    }
    const thumbnails = (await readDir(projectThumbnailDir)).map(p => p.name);
    const newThumbnails: {
      type: 'raw' | 'video';
      raw: string;
      thumbnailPath: string;
    }[] = [];
    // Identify ungenerated thumbnails
    for (const raw of raws) {
      const thumbnailFile = `${clean(raw).replace(/\..*$/, '')}.jpg`;
      const thumbnailPath = `${projectThumbnailDir}/${thumbnailFile}`; // tauri's join() slowed down this one line by like 10,000%
      const f = this.files[raw];
      if (f) {
        if (thumbnails.indexOf(thumbnailFile) < 0) {
          newThumbnails.push({ raw, thumbnailPath, type: 'raw' });
        } else {
          if (f.data.thumbnail.length === 0) {
            await this.setThumbnail(raw, convertFileSrc(thumbnailPath));
            this.emit('updatePhoto', f);
          }
          f.awaitingThumbnail = false;
        }
      }
    }
    for (const video of videos) {
      const thumbnailFile = `${clean(video).replace(/\..*$/, '')}.png`;
      const thumbnailPath = `${projectThumbnailDir}/${thumbnailFile}`;
      const f = this.files[video];
      if (f) {
        if (thumbnails.indexOf(thumbnailFile) < 0) {
          newThumbnails.push({ raw: video, thumbnailPath, type: 'video' });
        } else {
          if (f.data.thumbnail.length === 0) {
            await this.setThumbnail(video, convertFileSrc(thumbnailPath));
            this.emit('updatePhoto', f);
          }
          f.awaitingThumbnail = false;
        }
      }
    }
    // Generate new thumbnails
    for (const data of newThumbnails) {
      if (!(await exists(data.thumbnailPath))) {
        if (data.type === 'raw') {
          const convertOutput = await Command.create('magick', [
            data.raw,
            data.thumbnailPath,
          ]).execute();
          if (convertOutput.code !== 0) {
            console.error(convertOutput.stderr);
          }
          const resizeOutput = await Command.create('magick', [
            data.thumbnailPath,
            '-resize',
            '800x800',
            data.thumbnailPath,
          ]).execute();
          if (resizeOutput.code !== 0) {
            console.error(resizeOutput.stderr);
          }
        } else {
          const convertOutput = await Command.create('ffmpeg', [
            '-i',
            data.raw,
            '-ss',
            '00:00:01.00',
            '-vframes',
            '1',
            data.thumbnailPath,
          ]).execute();
          if (convertOutput.code !== 0) {
            console.error(convertOutput.stderr);
          }
        }
      }
      const f = this.files[data.raw];
      if (f) {
        if (f.data.thumbnail.length === 0) {
          await this.setThumbnail(data.raw, convertFileSrc(data.thumbnailPath));
          this.emit('updatePhoto', f);
        }
        f.awaitingThumbnail = false;
      }
      progress += 1;
      const p = Math.round((progress / newThumbnails.length) * 100);
      if (p > lastProgressInt) {
        this.thumbnailProgress = p;
        lastProgressInt = p;
        this.emit('thumbnailProgress', this.thumbnailProgress);
      }
    }
    this.generatingThumbnails = false;
  }

  /**
   * Gets a file.
   * @param name - The name of the file.
   * @returns The file object.
   */
  public getFile = (name: string) => {
    return this.files[name];
  }

  /**
   * Sets a photo's location.
   * @param photo - The target photo.
   * @param location - The location to set.
   */
  public setLocation = async (photo: string, location: string) => {
    if (this.files[photo]?.hasLocation) {
      const oldPos = this.files[photo].data.location;
      if (this.places[oldPos]) {
        this.places[oldPos].count -= 1;
      }
      if (this.locationMap[oldPos]) {
        const idx = this.locationMap[oldPos].findIndex(p => p.data.name === photo);
        if (idx >= 0) {
          this.locationMap[oldPos].splice(idx, 1);
        }
      }
    }
    const f = this.files[photo];
    if (f) {
      f.data.location = location;
      if (this.places[location]) {
        this.places[location].count += 1;
      }
      if (!this.locationMap[location]) {
        this.locationMap[location] = [];
      }
      this.locationMap[location].push(f);
      await this.database?.insert(f);
      this.emit('updatePhoto', f);
      this.emit('updateLocations');
    }
  }

  /**
   * Sets a photo's people.
   * @param photo - The target photo.
   * @param people - The people in the photo.
   */
  public setPeople = async (photo: string, people: string[]) => {
    const oldPeople = this.files[photo]?.people;
    oldPeople?.forEach(person => {
      if (this.peoplePhotoMap[person]) {
        const idx = this.peoplePhotoMap[person].findIndex(p => p.data.name === photo);
        if (idx >= 0) {
          this.peoplePhotoMap[person].splice(idx, 1);
        }
      }
      if (this.people[person]) {
        this.people[person].count -= 1;
      }
    });
    const f = this.files[photo];
    if (f) {
      f.people = people;
      people.forEach(id => {
        if (this.people[id]) {
          this.people[id].count += 1;
        }
        if (!this.peoplePhotoMap[id]) {
          this.peoplePhotoMap[id] = [];
        }
        this.peoplePhotoMap[id].push(f);
      });
      await this.database?.insert(f);
    }
  }

  /**
   * Creates a Place entry.
   * @param name - The name of the place.
   * @param pos - The latitude & longitude of the place.
   * @param layer - The layer the place belongs to.
   */
  public createPlace = async (name: string, pos: Position, category: PlaceType, layer: string) => {
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
    const prev = this.places[this.newestPlace];
    if (prev) {
      prev.isNewestPlace = false;
    }
    this.newestPlace = p.Id;
    this.places[p.Id] = p;
    await this.database?.insert(p);
    return p;
  };

  /**
   * Creates a Layer entry.
   * @param name - The name of the layer.
   */
  public createLayer = async (name: string) => {
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
    return Object.values(this.places).filter(p => p.data.layer === layer);
  }

  /**
   * Sets & saves a layer's color.
   * @param layer - The target layer.
   * @param color - The color to set.
   */
  public setLayerColor = async (layer: string, color: string) => {
    const l = this.layers[layer];
    if (l) {
      l.data.color = color;
      await this.database?.update(l);
    }
  }

  /**
   * Creates a shape.
   * @param type - The shape type.
   * @param points - The shape path.
   * @param layer - The layer the shape belongs to.
   * @param name - The name of the shape.
   */
  public createShape = async (type: ShapeType, points: Position[], layer: string, name: string) => {
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
  public setPlaceShape = async (place: string, shape: Shape) => {
    const p = this.places[place];
    if (p) {
      p.data.shape = shape.Id;
      await this.database?.update(p);
    }
  }

  /**
   * Updates a shape's path.
   * @param shape - The target shape.
   * @param path - The new path.
   */
  public setShapePath = async (shape: string, path: Position[]) => {
    const s = this.shapes[shape];
    if (s) {
      s.points = path;
      await this.database?.update(s);
    }
  }

  /**
   * Sets a shape's name.
   * @param shape - The target shape.
   * @param name - The new name.
   */
  public setShapeName = async (shape: string, name: string) => {
    const s = this.shapes[shape];
    if (s) {
      s.data.name = name;
      await this.database?.update(s);
    }
  }

  /**
   * Sets a place's name.
   * @param place - The target place.
   * @param name - The new name.
   */
  public setPlaceName = async (place: string, name: string) => {
    const p = this.places[place];
    if (p) {
      p.data.name = name;
      await this.database?.update(p);
    }
  }

  /**
   * Deletes a place.
   * @param place - The target place.
   */
  public deletePlace = async (place: string) => {
    const p = this.places[place];
    if (p) {
      await this.database?.delete(p);
      delete this.places[place];
    }
  }

  /**
   * Deletes a shape.
   * @param shape - The target shape.
   */
  public deleteShape = async (shape: string) => {
    const s = this.shapes[shape];
    if (s) {
      await this.database?.delete(s);
      delete this.shapes[shape];
    }
  }

  /**
   * Sets a place's layer.
   * @param place - The target place.
   * @param layer - The target layer.
   */
  public setPlaceLayer = async (place: string, layer: string) => {
    const p = this.places[place];
    if (p) {
      p.data.layer = layer;
      await this.database?.update(p);
    }
  }

  /**
   * Sets a shape's layer.
   * @param shape - The target shape.
   * @param layer - The target layer.
   */
  public setShapeLayer = async (shape: string, layer: string) => {
    const s = this.shapes[shape];
    if (s) {
      s.data.layer = layer;
      await this.database?.update(s);
    }
  }

  /**
   * Sets a place's tags.
   * @param place - The target place.
   * @param tags - The tags to set.
   */
  public setPlaceTags = async (place: string, tags: string[]) => {
    const p = this.places[place];
    if (p) {
      p.tags = tags;
      await this.database?.update(p);
    }
  }

  /**
   * Sets a place's notes.
   * @param place - The target place.
   * @param notes - The notes to set.
   */
  public setPlaceNotes = async (place: string, notes: string) => {
    const p = this.places[place];
    if (p) {
      p.data.notes = notes;
      await this.database?.update(p);
    }
  }

  /**
   * Sets a place's category.
   * @param place - The target place.
   * @param category - The category to set.
   */
  public setPlaceCategory = async (place: string, category: PlaceType) => {
    const p = this.places[place];
    if (p) {
      p.data.category = category;
      await this.database?.update(p);
    }
  }

  /**
   * Sets a place's position.
   * @param place - The target place.
   * @param position - The position to set.
   */
  public setPlacePosition = async (place: string, position: Position) => {
    const p = this.places[place];
    if (p) {
      p.data.lat = position.lat;
      p.data.lng = position.lng;
      await this.database?.update(p);
    }
  }

  /**
   * Update the calendar's focused date.
   * @param date - The date to focus on.
   */
  public setCalendarViewDate = (date: Date) => {
    this.calendarViewDate = date;
  }

  /**
   * Creates a new journal entry.
   * @param date - The date of the entry.
   * @param mood - The mood.
   * @param text - The entry text.
   * @param activities - The entry activities.
   * @param steps - The number of steps taken.
   */
  public createJournalEntry = async (
    date: string,
    mood: number,
    text: string,
    activities: Activity[],
    steps: number,
  ) => {
    if (!this.journals[date]) {
      const entry = new JournalEntry({
        date,
        mood,
        text,
        activities: activities.map(a => a.Id).join(','),
        steps,
        iv: '',
      });
      entry.activities = activities;
      this.journals[date] = entry;
      await this.database?.insert(entry);
    } else {
      this.journals[date].data = {
        mood,
        text,
        date,
        activities: activities.map(a => a.Id).join(','),
        steps,
        iv: this.journals[date].data.iv,
      };
      this.journals[date].activities = activities;
      await this.database?.update(this.journals[date]);
    }
    if (this.settings.encrypt) {
      await this.encryptJournalEntry(date);
      if (!this.encrypted) {
        this.journals[date].data.text = text;
      }
    }
    return this.journals[date];
  }

  /**
   * Sets a journal entry's mood.
   * @param date - The date of the entry.
   * @param mood - The mood to set.
   */
  public setEntryMood = async (date: string, mood: number) => {
    if (!this.journals[date]) {
      const entry = new JournalEntry({
        date,
        mood,
        text: '',
        activities: '',
        steps: 0,
        iv: '',
      });
      this.journals[date] = entry;
      await this.database?.insert(entry);
    } else {
      this.journals[date].data.mood = mood;
      await this.database?.update(this.journals[date]);
    }
    return this.journals[date];
  }

  /**
   * Sets a journal entry's text.
   * @param date - The date of the entry.
   * @param text - The entry text.
   */
  public setEntryText = async (date: string, text: string) => {
    if (!this.journals[date]) {
      const entry = new JournalEntry({
        date,
        mood: 2,
        text,
        activities: '',
        steps: 0,
        iv: '',
      });
      this.journals[date] = entry;
      await this.database?.insert(entry);
    } else {
      this.journals[date].data.text = text;
      await this.database?.update(this.journals[date]);
    }
    return this.journals[date];
  }

  /**
   * Creates a new activity.
   * @param name - The name of the activity.
   * @param icon - The icon for the activity.
   */
  public createActivity = async (name: string, icon: string) => {
    const a = new Activity({ name, icon });
    this.activities[a.Id] = a;
    await this.database?.insert(a);
    return a;
  }

  /**
   * Adds a person.
   * @param name - The name of the person.
   * @param notes - Initial notes for the person.
   * @param category - Category color.
   */
  public addPerson = async (name: string, notes: string, category: string) => {
    const p = new Person({
      name,
      photo: '',
      notes,
      category,
    });
    this.peopleMap[category]?.push(p);
    this.people[p.Id] = p;
    await this.database?.insert(p);
    return p;
  }

  /**
   * Updates a person.
   * @param name - The name of the person.
   * @param notes - Initial notes for the person.
   * @param category - Category color.
   */
  public updatePerson = async (id: string, name: string, notes: string, category: string) => {
    const p = this.people[id];
    if (p) {
      p.data = {
        ...p.data,
        name,
        notes,
        category,
      };
      await this.database?.update(p);
      return this.people[id];
    }
  }

  /**
   * Sets a person's profile photo.
   * @param person - The target person.
   * @param photo - The photo to set.
   */
  public setPersonPhoto = async (person: string, photo: string) => {
    const p = this.people[person];
    if (p) {
      p.data.photo = photo;
      await this.database?.update(p);
    }
  }

  /**
   * Adds a person category.
   * @param name - The name of the category.
   * @param color - The color of the category.
   */
  public addPersonCategory = async (name: string, color: string) => {
    const c = new PersonCategory({
      name,
      color,
    });
    await this.database?.insert(c);
    this.peopleCategories[c.Id] = c;
    this.peopleMap[c.Id] = [];
    return c;
  }

  /**
   * Hides a photo's thumbnail.
   * @param photo - The target photo.
   * @param value - If the thumbnail should be shown.
   */
  public setHideThumbnail = async (photo: string, value: boolean) => {
    const f = this.files[photo];
    if (f) {
      f.data.hideThumbnail = value;
      await this.database?.update(f);
      this.emit('updatePhoto', f);
    }
  }

  /**
   * Sets a photo's photographer
   * @param photo - The target photo.
   * @param value - The target person.
   */
  public setPhotographer = async (photo: string, value: string) => {
    const old = this.files[photo]?.data.photographer;
    if (old && this.people[old]) {
      this.people[old].photographerCount -= 1;
    }
    if (old && this.photographerMap[old]) {
      const idx = this.photographerMap[old].findIndex(p => p.data.name === photo);
      if (idx >= 0) {
        this.photographerMap[old].splice(idx, 1);
      }
    }
    const f = this.files[photo];
    if (f) {
      f.data.photographer = value;
      const p = this.people[value];
      if (p) {
        p.photographerCount += 1;
      }
      if (!this.photographerMap[value]) {
        this.photographerMap[value] = [];
      }
      this.photographerMap[value].push(f);
      await this.database?.update(f);
      this.emit('updatePhoto', f);
    }
  }

  /**
   * Sets the folder structure.
   * @param structure - The folder structure.
   */
  public setFolderStructure = (structure: FolderStructure) => {
    this.folder = structure;
  }

  /**
   * Sets the view mode.
   * @param mode - The view mode.
   */
  public setViewMode = (mode: number) => {
    this.viewMode = mode;
  }

  /**
   * Sets the sort mode.
   * @param mode - The sort mode.
   * @param dir - The sort direction.
   */
  public setSortMode(mode: number, dir: number) {
    this.sort = [mode, dir];
  }

  /**
   * Encrypts text.
   * @param text - The text to encrypt.
   * @param iv - If specified, the IV to use. Otherwise generates and returns a new one.
   */
  public async encrypt(text: string, iv = crypto.getRandomValues(new Uint8Array(12))) {
    return {
      text: ab2b64(
        await crypto.subtle.encrypt(
          {
            name: 'AES-GCM',
            iv,
          },
          this.key,
          new TextEncoder().encode(text),
        ),
      ),
      iv,
    };
  }

  /**
   * Decrypts text.
   * @param text - The encrypted text.
   * @param iv - The IV
   * @returns The decrypted text.
   */
  public async decrypt(text: string, iv: string) {
    return new TextDecoder().decode(
      await crypto.subtle.decrypt(
        {
          name: 'AES-GCM',
          iv: b642ab(iv),
        },
        this.key,
        b642ab(text),
      ),
    );
  }

  /**
   * Encrypts a single journal entry.
   * @param entry - The target entry.
   */
  public async encryptJournalEntry(entry: string) {
    const j = this.journals[entry];
    if (j) {
      const encrypted = await this.encrypt(j.data.text);
      j.data.text = encrypted.text;
      j.data.iv = ab2b64(encrypted.iv);
      await this.database?.update(j);
    }
  }

  /**
   * Encrypts a wiki page.
   * @param page - The target page.
   */
  public async encryptWikiPage(page: string) {
    const w = this.wikiPages[page];
    if (w) {
      const encrypted = await this.encrypt(w.data.content);
      w.data.content = encrypted.text;
      w.data.iv = ab2b64(encrypted.iv);
      w.data.name = (await this.encrypt(w.data.name, encrypted.iv)).text;
      await this.database?.update(w);
    }
  }

  /**
   * Encrypts all existing journal entries in the state & database.
   * @param password - The encryption password.
   */
  public encryptJournalEntries = async (password: string) => {
    if (!this.settings.encrypt) {
      await this.database?.insert(
        new Setting({
          setting: 'encrypt',
          value: true,
        }),
      );
      this.settings.encrypt = true;
      const total = Object.values(this.journals).length;
      let done = 0;
      let pw = password;
      if (pw.length < 128) {
        for (let i = pw.length; i < 16; i += 1) {
          pw += '0';
        }
      }
      this.key = await crypto.subtle.importKey(
        'raw',
        new TextEncoder().encode(pw),
        'AES-GCM',
        false,
        ['encrypt', 'decrypt'],
      );
      for (const entry of Object.values(this.journals)) {
        await this.encryptJournalEntry(entry.data.date);
        done += 1;
        this.emit('encryptionProgress', (done / total) * 100);
      }
      this.encrypted = true;
    }
  }

  /**
   * Decrypts all journal entries in the state (not the database)
   * @param password - The password to use.
   * @param save - If the decrypted entry should be written to the database.
   */
  public decryptJournalEntries = async (password: string, save = false) => {
    let pw = password;
    if (pw.length < 128) {
      for (let i = pw.length; i < 16; i += 1) {
        pw += '0';
      }
    }
    this.key = await crypto.subtle.importKey(
      'raw',
      new TextEncoder().encode(pw),
      'AES-GCM',
      false,
      ['encrypt', 'decrypt'],
    );
    for (const entry of Object.values(this.journals)) {
      const d = this.normalizeJournalDate(entry.data.date);
      const j = this.journals[d];
      if (j) {
        j.data.text = await this.decrypt(entry.data.text, entry.data.iv);
        if (save) {
          await this.database?.update(j);
        }
      }
    }
    for (const page of Object.values(this.wikiPages)) {
      const w = this.wikiPages[page.Id];
      if (w) {
        w.data.name = await this.decrypt(page.data.name, page.data.iv);
        w.data.content = await this.decrypt(page.data.content, page.data.iv);
        this.emit('updateWiki');
      }
    }
    this.encrypted = false;
    const r = this.settingsRecord.encrypted;
    if (save && r) {
      r.data.value = false;
      await this.database?.update(r);
    }
    this.emit('decrypted');
  }

  /**
   * Adds a new camera.
   * @param name - The name of the camera.
   */
  public addCamera = async (name: string) => {
    const c = new Camera({ name });
    this.cameras[c.Id] = c;
    await this.database?.insert(c);
  }

  /**
   * Sets a photo's camera.
   * @param photo - The target photo.
   * @param camera - The camera to set.
   */
  public setCamera = async (photo: string, camera: string) => {
    const old = this.files[photo]?.data.camera;
    if (old && old.length > 0 && this.cameras[old]) {
      this.cameras[old].count -= 1;
    }
    const f = this.files[photo];
    if (f) {
      f.data.camera = camera;
      const c = this.cameras[camera];
      if (c) {
        c.count += 1;
      }
      await this.database?.insert(f);
    }
  }

  /**
   * Toggles light/dark mode.
   */
  public toggleTheme = async () => {
    this.theme = !this.theme;
    this.emit('toggleTheme');
    if (this.settingsRecord.theme) {
      this.settingsRecord.theme.data.value = this.theme;
      await this.database?.update(this.settingsRecord.theme);
    } else {
      const s = new Setting({ setting: 'theme', value: this.theme });
      this.settingsRecord.theme = s;
      await this.database?.insert(this.settingsRecord.theme);
    }
  }

  private parseSearchTerms() {
    const terms: SearchTerm[] = [];
    this.query.forEach(term => {
      let matched = false;
      let negated = false;
      if (term[0] === '-') {
        negated = true;
        term = term.substring(1);
      }
      if (term.includes('=')) {
        const [target, value] = term.split('=');
        if (target && value && ['date', 'at', 'of', 'by'].indexOf(target) >= 0) {
          terms.push({
            type: 'rule',
            target,
            comparison: '=',
            value,
            negated,
          });
          matched = true;
        }
      } else if (term.includes('<')) {
        const [target, value] = term.split('<');
        if (target === 'date' && value) {
          terms.push({
            type: 'rule',
            target: 'date',
            comparison: '<',
            value,
            negated,
          });
          matched = true;
        }
      } else if (term.includes('>')) {
        const [target, value] = term.split('>');
        if (target === 'date' && value) {
          terms.push({
            type: 'rule',
            target: 'date',
            comparison: '>',
            value,
            negated,
          });
          matched = true;
        }
      } else if (term.includes(':')) {
        const [target, value] = term.split(':');
        if (target === 'is') {
          if (value === 'video') {
            terms.push({
              type: 'rule',
              comparison: 'is',
              value: 'video',
              negated,
            });
            matched = true;
          }
        } else if (target && value && ['of', 'by', 'at', 'only'].includes(target)) {
          terms.push({
            type: 'rule',
            comparison: target,
            value,
            negated,
          });
          matched = true;
        } else if (target === 'has' && value) {
          if (['location', 'rating', 'photographer', 'date'].indexOf(value) >= 0) {
            terms.push({
              type: 'rule',
              comparison: 'has',
              value,
              negated,
            });
            matched = true;
          }
        } else if (target === 'include' && value) {
          if (['duplicates'].indexOf(value) >= 0) {
            terms.push({
              type: 'rule',
              comparison: 'include',
              value,
              negated,
            });
          }
        }
      }
      if (!matched) {
        // Fallback to tag search
        terms.push({
          type: 'tag',
          value: term,
          negated,
        });
      }
    });
    return terms;
  }

  /**
   * Performs a search.
   * @param query - The query terms.
   */
  public search = (...query: string[]) => {
    if (query !== undefined) {
      this.query = query;
    }
    // TODO: Perform SELECT using non-tag fields
    let results = Object.values(this.files);
    // Filter results by tags
    results = results.filter(photo => this.checkFilter(photo));
    this.emit('search', results);
  }

  private findWikiPageByName(name: string) {
    return Object.values(this.wikiPages).find(p => p.data.name === name);
  }

  /**
   * Creates a new wiki page in the given path.
   * @param path - The path to create the page in.
   */
  public createWikiPage = async (path: string) => {
    let num = 1;
    if (path[0] === '/') {
      path = path.substring(1);
    }
    let conflicts = this.findWikiPageByName(`${path}/Untitled ${num}`) != undefined;
    while (conflicts) {
      num += 1;
      conflicts = this.findWikiPageByName(`${path}/Untitled ${num}`) != undefined;
    }
    const page = new WikiPage({
      name: `${path}/Untitled ${num}`,
      content: '',
      iv: '',
    });
    this.wikiPages[page.Id] = page;
    await this.database?.insert(page);
    if (this.settings.encrypt) {
      await this.encryptWikiPage(page.Id);
      const w = this.wikiPages[page.Id];
      if (!this.encrypted && w) {
        w.data.name = `${path}/Untitled ${num}`;
        w.data.content = '';
      }
    }
    this.emit('updateWiki');
  }

  /**
   * Update a wiki page's text.
   * @param path - The page to update.
   * @param content - The content to set.
   */
  public setWikiPageText = async (path: string, content: string) => {
    const w = this.wikiPages[path];
    if (w) {
      w.data.content = content;
      if (this.settings.encrypt) {
        const name = w.data.name;
        await this.encryptWikiPage(path);
        if (!this.encrypted) {
          w.data.name = name;
          w.data.content = content;
        }
      } else {
        await this.database?.update(w);
      }
    }
  }

  /**
   * Sets a wiki page's title.
   * @param page - The target page.
   * @param newTitle - The new title.
   */
  public setWikiPageTitle = async (page: string, newTitle: string) => {
    const w = this.wikiPages[page];
    if (w) {
      w.data.name = newTitle;
      if (this.settings.encrypt) {
        const content = w.data.content;
        await this.encryptWikiPage(page);
        if (!this.encrypted) {
          w.data.name = newTitle;
          w.data.content = content;
        }
      } else {
        await this.database?.update(w);
      }
    }
  }
}

const f = new FileStore();
Object.getOwnPropertyNames(Object.getPrototypeOf(f)).forEach(key => {
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
