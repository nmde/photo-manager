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
  return Uint8Array.from(atob(base64string), (c) => c.charCodeAt(0));
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
    this.emit('updatePhoto', this.files[photo]);
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
    this.emit('updatePhoto', this.files[photo]);
  }

  /**
   * Sets a photo's isDuplicate marker.
   * @param photo - The photo to set for.
   * @param isDuplicate - The duplicate marker.
   */
  public async setDuplicate(photo: string, isDuplicate: boolean) {
    this.files[photo].data.isDuplicate = isDuplicate;
    await this.database?.insert(this.files[photo]);
    this.emit('updatePhoto', this.files[photo]);
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
    const collectedPeople: string[] = [];
    let location = this.files[photo].data.location;
    let photographer = this.files[photo].data.photographer;
    this.getByGroup(group).forEach((photo) => {
      this.files[photo.data.name].tags.forEach((tag) => {
        if (collectedTags.indexOf(tag) < 0) {
          collectedTags.push(tag);
        }
      });
      this.files[photo.data.name].people.forEach((person) => {
        if (collectedPeople.indexOf(person) < 0) {
          collectedPeople.push(person);
        }
      });
      if (location.length === 0 && photo.data.location.length > 0) {
        location = photo.data.location;
      }
      if (photographer.length === 0 && photo.data.photographer.length > 0) {
        photographer = photo.data.photographer;
      }
    });
    if (!this.files[photo].hasLocation) {
      await this.setLocation(photo, location);
    }
    if (this.files[photo].data.photographer.length === 0) {
      await this.setPhotographer(photo, photographer);
    }
    await this.updatePeopleForGroup(photo, collectedPeople);
    await this.updateTagsForGroup(photo, collectedTags);
    this.emit('updatePhoto', this.files[photo]);
  }

  /**
   * Removes a photo from its group.
   * @param photo - The photo to remove from its group.
   */
  public async removeGroup(photo: string) {
    this.files[photo].data.photoGroup = '';
    await this.database?.insert(this.files[photo]);
    this.emit('updatePhoto', this.files[photo]);
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
    this.emit('updatePhoto', this.files[photo]);
    // TODO: inform of newly created tags
  }

  /**
   * Updates people for photo groups.
   * @param photo - The base photo.
   * @param p - The list of people.
   */
  public async updatePeopleForGroup(photo: string, p: string[]) {
    p.forEach((person) => {
      if (this.files[photo].group === undefined || this.files[photo].firstInGroup) {
        if (this.files[photo].people.indexOf(person) < 0) {
          this.people[person].count += 1;
        }
      }
    });
    if (this.files[photo].group === undefined || this.files[photo].firstInGroup) {
      this.files[photo].people.forEach((person) => {
        if (p.indexOf(person) < 0) {
          this.people[person].count -= 1;
        }
      });
    }
    this.files[photo].people = p;
    await this.database?.insert(this.files[photo]);
    this.emit('updatePhoto', this.files[photo]);
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
    this.emit('updatePhoto', this.files[photo]);
  }

  /**
   * Sets a photo's description.
   * @param photo - The photo.
   * @param description - The description to set.
   */
  public async setDescription(photo: string, description: string) {
    this.files[photo].data.description = description;
    await this.database?.insert(this.files[photo]);
    this.emit('updatePhoto', this.files[photo]);
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

  private normalizeJournalDate(date: string) {
    return formatDate(new Date(date));
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
        this.locationMap[place.Id] = [];
        place.tags.forEach((tag) => {
          if (tagList.indexOf(tag) < 0) {
            tagList.push(tag);
          }
        });
      });
      (await this.database.selectAll(PersonCategory)).forEach((pcat) => {
        this.peopleCategories[pcat.Id] = pcat;
        this.peopleMap[pcat.Id] = [];
      });
      (await this.database.selectAll(Person)).forEach((person) => {
        this.peopleMap[person.data.category].push(person);
        this.people[person.Id] = person;
      });
      (await this.database.selectAll(Camera)).forEach((camera) => {
        this.cameras[camera.Id] = camera;
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
          if (!this.locationMap[photo.data.location]) {
            this.locationMap[photo.data.location] = [];
          }
          this.locationMap[photo.data.location].push(photo);
        }
        photo.people.forEach((id) => {
          if (this.people[id]) {
            this.people[id].count += 1;
          }
          if (!this.peoplePhotoMap[id]) {
            this.peoplePhotoMap[id] = [];
          }
          this.peoplePhotoMap[id].push(photo);
        });
        if (photo.data.photographer !== undefined && this.people[photo.data.photographer]) {
          this.people[photo.data.photographer].photographerCount += 1;
          if (!this.photographerMap[photo.data.photographer]) {
            this.photographerMap[photo.data.photographer] = [];
          }
          this.photographerMap[photo.data.photographer].push(photo);
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
        if (photo.data.camera && photo.data.camera.length > 0 && this.cameras[photo.data.camera]) {
          this.cameras[photo.data.camera].count += 1;
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
      (await this.database.selectAll(Activity)).forEach((activity) => {
        this.activities[activity.Id] = activity;
      });
      (await this.database.selectAll(Setting)).forEach((setting) => {
        this.settings[setting.data.setting] = setting.data.value;
        this.settingsRecord[setting.data.setting] = setting;
        if (setting.data.setting === 'encrypt') {
          this.encrypted = setting.data.value;
        } else if (setting.data.setting === 'theme') {
          this.theme = setting.data.value;
          if (this.theme) {
            this.emit('toggleTheme');
          }
        }
      });
      (await this.database.selectAll(JournalEntry)).forEach((entry) => {
        const d = this.normalizeJournalDate(entry.data.date);
        this.journals[d] = entry;
        if (entry.data.activities.length > 0) {
          this.journals[d].activities = entry.data.activities
            .split(',')
            .map((a) => this.activities[a]);
        }
      });
      (await this.database.selectAll(WikiPage)).forEach((page) => {
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
    if (this.files[photo].data.date.length > 0) {
      const oldDate = formatDate(this.files[photo].date);
      if (this.dateMap[oldDate]) {
        const idx = this.dateMap[oldDate].findIndex((p) => p.data.name === photo);
        if (idx >= 0) {
          this.dateMap[oldDate].splice(idx, 1);
        }
      }
    }
    this.files[photo].data.date = date;
    const d = formatDate(this.files[photo].date);
    if (!this.dateMap[d]) {
      this.dateMap[d] = [];
    }
    if (this.files[photo].date < this.firstDate) {
      this.firstDate = this.files[photo].date;
    }
    if (this.files[photo].date > this.lastDate) {
      this.lastDate = this.files[photo].date;
    }
    this.dateMap[d].push(this.files[photo]);
    await this.database?.insert(this.files[photo]);
    this.emit('updatePhoto', this.files[photo]);
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
    this.emit('updatePhoto', this.files[photo]);
    this.emit('validationUpdate', photo);
  }

  /**
   * Checks a single photo to see if it matches the current search query.
   * @param photo - The photo to check.
   */
  public checkFilter(photo: Photo) {
    if (photo.hidden) {
      return false;
    }
    const terms = this.parseSearchTerms();
    let satisfiesRules = true;
    const rules = terms.filter((t) => t.type === 'rule');
    rules.forEach((rule) => {
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
        const mappedPeople = photo.people.map((p) => this.people[p].data.name);
        if (rule.negated) {
          satisfiesRules = satisfiesRules && mappedPeople.indexOf(rule.value) < 0;
        } else {
          satisfiesRules = satisfiesRules && mappedPeople.indexOf(rule.value) >= 0;
        }
      } else if (rule.comparison === 'only') {
        const mappedPeople = photo.people.map((p) => this.people[p].data.name);
        if (rule.negated) {
          satisfiesRules =
            satisfiesRules && (mappedPeople.indexOf(rule.value) < 0 || mappedPeople.length > 1);
        } else {
          satisfiesRules =
            satisfiesRules && mappedPeople.indexOf(rule.value) === 0 && mappedPeople.length === 1;
        }
      } else if (rule.comparison === 'by') {
        if (photo.data.photographer.length > 0) {
          const name = this.people[photo.data.photographer].data.name;
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
          const name = this.places[photo.data.location].data.name;
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
    const tags = terms.filter((t) => t.type === 'tag');
    let hasAllTags = true;
    tags.forEach((tag) => {
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
  public async generateThumbnails(raws: string[], videos: string[]) {
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
    const thumbnails = (await readDir(projectThumbnailDir)).map((p) => p.name);
    const newThumbnails: {
      type: 'raw' | 'video';
      raw: string;
      thumbnailPath: string;
    }[] = [];
    // Identify ungenerated thumbnails
    for (const raw of raws) {
      const thumbnailFile = `${clean(raw).replace(/\..*$/, '')}.jpg`;
      const thumbnailPath = `${projectThumbnailDir}/${thumbnailFile}`; // tauri's join() slowed down this one line by like 10,000%
      if (thumbnails.indexOf(thumbnailFile) < 0) {
        newThumbnails.push({ raw, thumbnailPath, type: 'raw' });
      } else {
        if (this.files[raw].data.thumbnail.length === 0) {
          await this.setThumbnail(raw, convertFileSrc(thumbnailPath));
          this.emit('updatePhoto', this.files[raw]);
        }
        this.files[raw].awaitingThumbnail = false;
      }
    }
    for (const video of videos) {
      const thumbnailFile = `${clean(video).replace(/\..*$/, '')}.png`;
      const thumbnailPath = `${projectThumbnailDir}/${thumbnailFile}`;
      if (thumbnails.indexOf(thumbnailFile) < 0) {
        newThumbnails.push({ raw: video, thumbnailPath, type: 'video' });
      } else {
        if (this.files[video].data.thumbnail.length === 0) {
          await this.setThumbnail(video, convertFileSrc(thumbnailPath));
          this.emit('updatePhoto', this.files[video]);
        }
        this.files[video].awaitingThumbnail = false;
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
      if (this.files[data.raw].data.thumbnail.length === 0) {
        await this.setThumbnail(data.raw, convertFileSrc(data.thumbnailPath));
        this.emit('updatePhoto', this.files[data.raw]);
      }
      this.files[data.raw].awaitingThumbnail = false;
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
  public getFile(name: string) {
    return this.files[name];
  }

  /**
   * Sets a photo's location.
   * @param photo - The target photo.
   * @param location - The location to set.
   */
  public async setLocation(photo: string, location: string) {
    if (this.files[photo].hasLocation) {
      const oldPos = this.files[photo].data.location;
      if (this.places[oldPos]) {
        this.places[this.files[photo].data.location].count -= 1;
      }
      if (this.locationMap[oldPos]) {
        const idx = this.locationMap[oldPos].findIndex((p) => p.data.name === photo);
        if (idx >= 0) {
          this.locationMap[oldPos].splice(idx, 1);
        }
      }
    }
    this.files[photo].data.location = location;
    if (this.places[location]) {
      this.places[location].count += 1;
    }
    if (!this.locationMap[location]) {
      this.locationMap[location] = [];
    }
    this.locationMap[location].push(this.files[photo]);
    await this.database?.insert(this.files[photo]);
    this.emit('updatePhoto', this.files[photo]);
    this.emit('updateLocations');
  }

  /**
   * Sets a photo's people.
   * @param photo - The target photo.
   * @param people - The people in the photo.
   */
  public async setPeople(photo: string, people: string[]) {
    const oldPeople = this.files[photo].people;
    oldPeople.forEach((person) => {
      if (this.peoplePhotoMap[person]) {
        const idx = this.peoplePhotoMap[person].findIndex((p) => p.data.name === photo);
        if (idx >= 0) {
          this.peoplePhotoMap[person].splice(idx, 1);
        }
      }
      if (this.people[person]) {
        this.people[person].count -= 1;
      }
    });
    this.files[photo].people = people;
    people.forEach((id) => {
      this.people[id].count += 1;
      if (!this.peoplePhotoMap[id]) {
        this.peoplePhotoMap[id] = [];
      }
      this.peoplePhotoMap[id].push(this.files[photo]);
    });
    await this.database?.insert(this.files[photo]);
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

  /**
   * Creates a new journal entry.
   * @param date - The date of the entry.
   * @param mood - The mood.
   * @param text - The entry text.
   * @param activities - The entry activities.
   * @param steps - The number of steps taken.
   */
  public async createJournalEntry(
    date: string,
    mood: number,
    text: string,
    activities: Activity[],
    steps: number,
  ) {
    if (!this.journals[date]) {
      const entry = new JournalEntry({
        date,
        mood,
        text,
        activities: activities.map((a) => a.Id).join(','),
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
        activities: activities.map((a) => a.Id).join(','),
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
  public async setEntryMood(date: string, mood: number) {
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
  public async setEntryText(date: string, text: string) {
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
  public async createActivity(name: string, icon: string) {
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
  public async addPerson(name: string, notes: string, category: string) {
    const p = new Person({
      name,
      photo: '',
      notes,
      category,
    });
    this.peopleMap[category].push(p);
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
  public async updatePerson(id: string, name: string, notes: string, category: string) {
    this.people[id].data = {
      ...this.people[id].data,
      name,
      notes,
      category,
    };
    await this.database?.update(this.people[id]);
    return this.people[id];
  }

  /**
   * Sets a person's profile photo.
   * @param person - The target person.
   * @param photo - The photo to set.
   */
  public async setPersonPhoto(person: string, photo: string) {
    this.people[person].data.photo = photo;
    await this.database?.update(this.people[person]);
  }

  /**
   * Adds a person category.
   * @param name - The name of the category.
   * @param color - The color of the category.
   */
  public async addPersonCategory(name: string, color: string) {
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
  public async setHideThumbnail(photo: string, value: boolean) {
    this.files[photo].data.hideThumbnail = value;
    await this.database?.update(this.files[photo]);
    this.emit('updatePhoto', this.files[photo]);
  }

  /**
   * Sets a photo's photographer
   * @param photo - The target photo.
   * @param value - The target person.
   */
  public async setPhotographer(photo: string, value: string) {
    const old = this.files[photo].data.photographer;
    if (old && this.people[old]) {
      this.people[old].photographerCount -= 1;
    }
    if (this.photographerMap[old]) {
      const idx = this.photographerMap[old].findIndex((p) => p.data.name === photo);
      if (idx >= 0) {
        this.photographerMap[old].splice(idx, 1);
      }
    }
    this.files[photo].data.photographer = value;
    this.people[value].photographerCount += 1;
    if (!this.photographerMap[value]) {
      this.photographerMap[value] = [];
    }
    this.photographerMap[value].push(this.files[photo]);
    await this.database?.update(this.files[photo]);
    this.emit('updatePhoto', this.files[photo]);
  }

  /**
   * Sets the folder structure.
   * @param structure - The folder structure.
   */
  public setFolderStructure(structure: FolderStructure) {
    this.folder = structure;
  }

  /**
   * Sets the view mode.
   * @param mode - The view mode.
   */
  public setViewMode(mode: number) {
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
    const encrypted = await this.encrypt(this.journals[entry].data.text);
    this.journals[entry].data.text = encrypted.text;
    this.journals[entry].data.iv = ab2b64(encrypted.iv);
    await this.database?.update(this.journals[entry]);
  }

  /**
   * Encrypts a wiki page.
   * @param page - The target page.
   */
  public async encryptWikiPage(page: string) {
    const encrypted = await this.encrypt(this.wikiPages[page].data.content);
    this.wikiPages[page].data.content = encrypted.text;
    this.wikiPages[page].data.iv = ab2b64(encrypted.iv);
    this.wikiPages[page].data.name = (
      await this.encrypt(this.wikiPages[page].data.name, encrypted.iv)
    ).text;;
    await this.database?.update(this.wikiPages[page]);
  }

  /**
   * Encrypts all existing journal entries in the state & database.
   * @param password - The encryption password.
   */
  public async encryptJournalEntries(password: string) {
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
  public async decryptJournalEntries(password: string, save = false) {
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
      this.journals[d].data.text = await this.decrypt(entry.data.text, entry.data.iv);
      if (save) {
        await this.database?.update(this.journals[d]);
      }
    }
    for (const page of Object.values(this.wikiPages)) {
      this.wikiPages[page.Id].data.name = await this.decrypt(page.data.name, page.data.iv);;
      this.wikiPages[page.Id].data.content = await this.decrypt(
        page.data.content,
        page.data.iv,
      );
      this.emit('updateWiki');
    }
    this.encrypted = false;
    if (save) {
      this.settingsRecord.encrypted.data.value = false;
      await this.database?.update(this.settingsRecord.encrypted);
    }
    this.emit('decrypted');
  }

  /**
   * Adds a new camera.
   * @param name - The name of the camera.
   */
  public async addCamera(name: string) {
    const c = new Camera({ name });
    this.cameras[c.Id] = c;
    await this.database?.insert(c);
  }

  /**
   * Sets a photo's camera.
   * @param photo - The target photo.
   * @param camera - The camera to set.
   */
  public async setCamera(photo: string, camera: string) {
    const old = this.files[photo].data.camera;
    if (old && old.length > 0 && this.cameras[old]) {
      this.cameras[old].count -= 1;
    }
    this.files[photo].data.camera = camera;
    this.cameras[camera].count += 1;
    await this.database?.insert(this.files[photo]);
  }

  /**
   * Toggles light/dark mode.
   */
  public async toggleTheme() {
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
    this.query.forEach((term) => {
      let matched = false;
      let negated = false;
      if (term[0] === '-') {
        negated = true;
        term = term.substring(1);
      }
      if (term.includes('=')) {
        const split = term.split('=');
        if (['date', 'at', 'of', 'by'].indexOf(split[0]) >= 0) {
          terms.push({
            type: 'rule',
            target: split[0],
            comparison: '=',
            value: split[1],
            negated,
          });
          matched = true;
        }
      } else if (term.includes('<')) {
        const split = term.split('<');
        if (split[0] === 'date') {
          terms.push({
            type: 'rule',
            target: 'date',
            comparison: '<',
            value: split[1],
            negated,
          });
          matched = true;
        }
      } else if (term.includes('>')) {
        const split = term.split('>');
        if (split[0] === 'date') {
          terms.push({
            type: 'rule',
            target: 'date',
            comparison: '>',
            value: split[1],
            negated,
          });
          matched = true;
        }
      } else if (term.includes(':')) {
        const split = term.split(':');
        if (split[0] === 'is') {
          if (split[1] === 'video') {
            terms.push({
              type: 'rule',
              comparison: 'is',
              value: 'video',
              negated,
            });
            matched = true;
          }
        } else if (['of', 'by', 'at', 'only'].includes(split[0])) {
          terms.push({
            type: 'rule',
            comparison: split[0],
            value: split[1],
            negated,
          });
          matched = true;
        } else if (split[0] === 'has') {
          if (['location', 'rating', 'photographer', 'date'].indexOf(split[1]) >= 0) {
            terms.push({
              type: 'rule',
              comparison: 'has',
              value: split[1],
              negated,
            });
            matched = true;
          }
        } else if (split[0] === 'include') {
          if (['duplicates'].indexOf(split[1]) >= 0) {
            terms.push({
              type: 'rule',
              comparison: 'include',
              value: split[1],
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
  public async search(...query: string[]) {
    if (query !== undefined) {
      this.query = query;
    }
    // TODO: Perform SELECT using non-tag fields
    let results = Object.values(this.files);
    // Filter results by tags
    results = results.filter((photo) => this.checkFilter(photo));
    this.emit('search', results);
  }

  private findWikiPageByName(name: string) {
    return Object.values(this.wikiPages).find((p) => p.data.name === name);
  }

  /**
   * Creates a new wiki page in the given path.
   * @param path - The path to create the page in.
   */
  public async createWikiPage(path: string) {
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
      if (!this.encrypted) {
        this.wikiPages[page.Id].data.name = `${path}/Untitled ${num}`;
        this.wikiPages[page.Id].data.content = '';
      }
    }
    this.emit('updateWiki');
  }

  /**
   * Update a wiki page's text.
   * @param path - The page to update.
   * @param content - The content to set.
   */
  public async setWikiPageText(path: string, content: string) {
    this.wikiPages[path].data.content = content;
    if (this.settings.encrypt) {
      const name = this.wikiPages[path].data.name;
      await this.encryptWikiPage(path);
      if (!this.encrypted) {
        this.wikiPages[path].data.name = name;
        this.wikiPages[path].data.content = content;
      }
    } else {
      await this.database?.update(this.wikiPages[path]);
    }
  }

  /**
   * Sets a wiki page's title.
   * @param page - The target page.
   * @param newTitle - The new title.
   */
  public async setWikiPageTitle(page: string, newTitle: string) {
    this.wikiPages[page].data.name = newTitle;
    if (this.settings.encrypt) {
      const content = this.wikiPages[page].data.content;
      await this.encryptWikiPage(page);
      if (!this.encrypted) {
        this.wikiPages[page].data.name = newTitle;
        this.wikiPages[page].data.content = content;
      }
    } else {
      await this.database?.update(this.wikiPages[page]);
    }
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
