import type { PlaceType, Position } from '../classes/Map';
import { invoke } from '@tauri-apps/api/core';
import { EventEmitter } from 'ee-ts';
import { v4 as uuid } from 'uuid';
import { decrypt } from '@/util/encrypt';
import { Activity } from '../classes/Activity';
import { Camera } from '../classes/Camera';
import { Graph } from '../classes/Graph';
import { GraphNode } from '../classes/GraphNode';
import { Group } from '../classes/Group';
import { JournalEntry } from '../classes/JournalEntry';
import { Layer } from '../classes/Layer';
import { Person } from '../classes/Person';
import { PersonCategory } from '../classes/PersonCategory';
import { Photo } from '../classes/Photo';
import { Place } from '../classes/Place';
import { Setting, type SettingKey } from '../classes/Setting';
import { Shape, type ShapeType } from '../classes/Shape';
import { Tag } from '../classes/Tag';
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

export function formatDate(date: Date) {
  return `${date.getFullYear().toString()}-${(date.getMonth() + 1).toString()}-${date
    .getDate()
    .toString()}`;
}
class FileStore extends EventEmitter<{
  updatePhoto: (photo?: Photo) => void;
  updateLocations: () => void;
  saving: (value: boolean) => void;
  saveError: () => void;
  thumbnailProgress: (progress: number) => void;
  validationUpdate: (photo: string) => void;
  encryptionProgress: (progress: number) => void;
  decrypted: () => void;
  toggleTheme: () => void;
  search: (results: Photo[]) => void;
  updateWiki: () => void;
}> {
  public activities: Record<string, Activity> = {};

  public advTags: Tag[] = [];

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

  public places: Record<string, Place> = {};

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

  public settings: {
    [key in SettingKey]: string;
  } = {
    encrypt: 'false',
    theme: 'false',
  };

  public encrypted = false;

  public cameras: Record<string, Camera> = {};

  public theme = false;

  public firstDate = new Date();

  public lastDate = new Date();

  public query: string[] = [];

  public wikiPages: Record<string, WikiPage> = {};

  private newestPlace = '';

  private settingsRecord: Record<string, Setting> = {};

  private key!: CryptoKey;

  /**
   * Adds a group.
   * @param name - The name of the group.
   */
  public addGroup = async (name: string) => {
    const id = uuid();
    const g = new Group(id, name);
    this.groups.push(g);
    this.groupNames.push(name);
    await invoke('create_group', {
      id,
      name,
    });
  };

  /**
   * Gets all photos in a group.
   * @param group - The group to get photos from.
   */
  public getByGroup = (group: string) => Object.values(this.files).filter(p => p.group === group);

  /**
   * Sets a photo's group.
   * @param photo - The photo to set.
   * @param group - The group to set.
   */
  public setGroup = async (photo: string, group?: string) => {
    const f = this.files[photo];
    if (f) {
      if (group === undefined) {
        await f.setGroup('');
        return;
      }
      await f.setGroup(group);
      const collectedTags: string[] = [];
      const collectedPeople: string[] = [];
      let location = this.files[photo]?.location;
      let photographer = this.files[photo]?.photographer;
      for (const photo of this.getByGroup(group)) {
        const f2 = this.files[photo.name];
        if (f2) {
          for (const tag of f2.tags) {
            if (!collectedTags.includes(tag)) {
              collectedTags.push(tag);
            }
          }
          for (const person of f2.people) {
            if (!collectedPeople.includes(person)) {
              collectedPeople.push(person);
            }
          }
        }
        if (location?.length === 0 && photo.location.length > 0) {
          location = photo.location;
        }
        if (photographer?.length === 0 && photo.photographer.length > 0) {
          photographer = photo.photographer;
        }
      }
      if (!this.files[photo]?.hasLocation && location) {
        await this.setLocation(photo, location);
      }
      if (this.files[photo]?.photographer.length === 0 && photographer) {
        await this.setPhotographer(photo, photographer);
      }
      await this.updatePeopleForGroup(photo, collectedPeople);
      await this.updateTagsForGroup(photo, collectedTags);
      this.emit('updatePhoto', f);
    }
  };

  /**
   * Removes a photo from its group.
   * @param photo - The photo to remove from its group.
   */
  public removeGroup = async (photo: string) => {
    await this.files[photo]?.setGroup('');
    this.emit('updatePhoto', this.files[photo]);
  };

  /**
   * Updates tags for photo groups.
   * @param photo - The base photo.
   * @param t - The list of tags.
   */
  public updateTagsForGroup = async (photo: string, t: string[]) => {
    const newTags: string[] = [];
    for (const tag of t) {
      if (this.files[photo]?.group === undefined || this.files[photo].firstInGroup) {
        if (!this.tagCounts[tag]) {
          this.tagCounts[tag] = 0;
        }
        if (this.files[photo] && !this.files[photo].tags.includes(tag)) {
          this.tagCounts[tag] += 1;
        }
      }
      if (!this.tags.includes(tag)) {
        this.tags.push(tag);
        newTags.push(tag);
      }
    }
    if (
      this.files[photo] &&
      (this.files[photo].group === undefined || this.files[photo].firstInGroup)
    ) {
      for (const tag of this.files[photo].tags) {
        if (!t.includes(tag) && this.tagCounts[tag]) {
          this.tagCounts[tag] -= 1;
          if (this.tagCounts[tag] <= 0) {
            delete this.tagCounts[tag];
            this.tags.splice(this.tags.indexOf(tag), 1);
          }
        }
      }
    }
    await this.files[photo]?.setTags(t);
    this.sortTags();
    this.emit('updatePhoto', this.files[photo]);
    // TODO: inform of newly created tags
  };

  /**
   * Updates people for photo groups.
   * @param photo - The base photo.
   * @param p - The list of people.
   */
  public async updatePeopleForGroup(photo: string, p: string[]) {
    const f = this.files[photo];
    if (f) {
      for (const person of p) {
        if (
          (f.group === undefined || f.firstInGroup) &&
          !f.people.includes(person) &&
          this.people[person]
        ) {
          this.people[person].count += 1;
        }
      }
      if (f.group === undefined || f.firstInGroup) {
        for (const person of f.people) {
          if (!p.includes(person) && this.people[person]) {
            this.people[person].count -= 1;
          }
        }
      }
      await f.setPeople(p);
      this.emit('updatePhoto', f);
    }
  }

  /**
   * Adds new tags to the master list.
   * @param t - The tags to apply.
   */
  public updateTags = (t: string[]) => {
    const newTags: string[] = [];
    for (const tag of t) {
      if (!this.tags.includes(tag)) {
        this.tags.push(tag);
        newTags.push(tag);
      }
    }
    this.sortTags();
  };

  /**
   * Sets a photo's title.
   * @param photo - The photo.
   * @param title - The title to set.
   */
  public setTitle = async (photo: string, title: string) => {
    await this.files[photo]?.setTitle(title);
    this.emit('updatePhoto', this.files[photo]);
  };

  /**
   * Sets a photo's description.
   * @param photo - The photo.
   * @param description - The description to set.
   */
  public setDescription = async (photo: string, description: string) => {
    await this.files[photo]?.setDescription(description);
    this.emit('updatePhoto', this.files[photo]);
  };

  /**
   * Loads photos from the database.
   */
  // eslint-disable-next-line complexity
  public loadPhotos = async () => {
    this.files = {};
    this.advTags = (
      await invoke<
        {
          id: string;
          name: string;
          color: string;
          prereqs: string;
          coreqs: string;
          incompatible: string;
        }[]
      >('get_tags')
    ).map(
      ({ id, name, color, prereqs, coreqs, incompatible }) =>
        new Tag(id, name, color, prereqs, coreqs, incompatible),
    );
    const tagList: string[] = [];
    const encounteredGroups: string[] = [];
    console.log('reading places');
    for (const place of (
      await invoke<
        {
          id: string;
          name: string;
          lat: number;
          lng: number;
          layer: string;
          category: PlaceType;
          shape: string;
          tags: string;
          notes: string;
        }[]
      >('get_places')
    ).map(
      ({ id, name, lat, lng, layer, category, shape, tags, notes }) =>
        new Place(id, name, lat, lng, layer, category, shape, tags, notes),
    )) {
      this.places[place.id] = place;
      this.locationMap[place.id] = [];
      for (const tag of place.tags) {
        if (!tagList.includes(tag)) {
          tagList.push(tag);
        }
      }
    }
    for (const pcat of (
      await invoke<
        {
          id: string;
          name: string;
          color: string;
        }[]
      >('get_person_categories')
    ).map(({ id, name, color }) => new PersonCategory(id, name, color))) {
      this.peopleCategories[pcat.id] = pcat;
      this.peopleMap[pcat.id] = [];
    }
    for (const person of (
      await invoke<{ id: string; name: string; photo: string; notes: string; category: string }[]>(
        'get_people',
      )
    ).map(({ id, name, photo, notes, category }) => new Person(id, name, photo, notes, category))) {
      this.peopleMap[person.category]?.push(person);
      this.people[person.id] = person;
    }
    for (const camera of (await invoke<{ id: string; name: string }[]>('get_cameras')).map(
      ({ id, name }) => new Camera(id, name),
    )) {
      this.cameras[camera.id] = camera;
    }
    const raws: Photo[] = [];
    for (const photo of (
      await invoke<
        {
          id: string;
          name: string;
          path: string;
          title: string;
          description: string;
          tags: string;
          is_duplicate: number;
          rating: number;
          location: string;
          thumbnail: string;
          video: number;
          photo_group: string;
          date: string;
          raw: number;
          people: string;
          hide_thumbnail: number;
          photographer: string;
          camera: string;
        }[]
      >('get_photos')
    ).map(
      ({
        id,
        name,
        path,
        title,
        description,
        tags,
        is_duplicate,
        rating,
        location,
        thumbnail,
        video,
        photo_group,
        date,
        raw,
        people,
        hide_thumbnail,
        photographer,
        camera,
      }) =>
        new Photo(
          id,
          name,
          path,
          title,
          description,
          location,
          tags,
          is_duplicate === 1,
          thumbnail,
          rating,
          video === 1,
          photo_group,
          date,
          raw === 1,
          people,
          hide_thumbnail === 1,
          photographer,
          camera,
        ),
    )) {
      this.files[photo.name] = photo;
      let firstInGroup = false;
      if (photo.group && !encounteredGroups.includes(photo.group)) {
        photo.firstInGroup = true;
        firstInGroup = true;
        encounteredGroups.push(photo.group);
      }
      if (photo.group === undefined || firstInGroup) {
        this.photoCount += 1;
        for (const tag of photo.tags) {
          if (!tagList.includes(tag)) {
            tagList.push(tag);
          }
          if (!this.tagCounts[tag]) {
            this.tagCounts[tag] = 0;
          }
          this.tagCounts[tag] += 1;
        }
      }
      const p = this.places[photo.location];
      if (photo.hasLocation && p) {
        p.count += 1;
        if (!this.locationMap[photo.location]) {
          this.locationMap[photo.location] = [];
        }
        this.locationMap[photo.location]?.push(photo);
      }
      for (const id of photo.people) {
        if (this.people[id]) {
          this.people[id].count += 1;
        }
        if (!this.peoplePhotoMap[id]) {
          this.peoplePhotoMap[id] = [];
        }
        this.peoplePhotoMap[id].push(photo);
      }
      const p2 = this.people[photo.photographer];
      if (photo.photographer.length > 0 && p2) {
        p2.photographerCount += 1;
        if (!this.photographerMap[photo.photographer]) {
          this.photographerMap[photo.photographer] = [];
        }
        this.photographerMap[photo.photographer]?.push(photo);
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
      const c = this.cameras[photo.camera];
      if (photo.camera && photo.camera.length > 0 && c) {
        c.count += 1;
      }
      this.validateTags(photo.name);
      if (photo.raw) {
        raws.push(photo);
      }
    }
    this.groupRaws(raws);
    this.groups = (await invoke<{ id: string; name: string }[]>('get_groups')).map(
      ({ id, name }) => new Group(id, name),
    );
    this.groupNames = this.groups.map(g => g.name);
    for (const layer of (
      await invoke<{ id: string; name: string; color: string }[]>('get_layers')
    ).map(({ id, name, color }) => new Layer(id, name, color))) {
      this.layers[layer.id] = layer;
    }
    for (const shape of (
      await invoke<
        { id: string; shape_type: ShapeType; points: string; layer: string; name: string }[]
      >('get_shapes')
    ).map(
      ({ id, shape_type, points, layer, name }) => new Shape(id, shape_type, points, layer, name),
    )) {
      this.shapes[shape.id] = shape;
    }
    for (const activity of (
      await invoke<{ id: string; name: string; icon: string }[]>('get_activities')
    ).map(({ id, name, icon }) => new Activity(id, icon, name))) {
      this.activities[activity.id] = activity;
    }
    for (const setting of (
      await invoke<{ id: string; setting: SettingKey; value: string }[]>('get_settings')
    ).map(({ id, setting, value }) => new Setting(id, setting, value))) {
      this.settings[setting.setting] = setting.value;
      this.settingsRecord[setting.setting] = setting;
      if (setting.setting === 'encrypt' && typeof setting.value === 'boolean') {
        this.encrypted = setting.value;
      } else if (setting.setting === 'theme' && typeof setting.value === 'boolean') {
        this.theme = setting.value;
        if (this.theme) {
          this.emit('toggleTheme');
        }
      }
    }
    for (const entry of (
      await invoke<
        {
          id: string;
          date: string;
          mood: number;
          text: string;
          activities: string;
          steps: number;
          iv: string;
        }[]
      >('get_journals')
    ).map(
      ({ id, date, mood, text, activities, steps, iv }) =>
        new JournalEntry(id, date, mood, text, activities, steps, iv),
    )) {
      const d = this.normalizeJournalDate(entry.date);
      this.journals[d] = entry;
      if (entry.activities.length > 0) {
        this.journals[d].activities = entry.activitiesStr
          .split(',')
          .map(a => this.activities[a])
          .filter(a => a !== undefined);
      }
    }
    for (const page of (
      await invoke<{ id: string; name: string; content: string; iv: string }[]>('get_wiki_pages')
    ).map(({ id, name, content, iv }) => new WikiPage(id, name, content, iv))) {
      this.wikiPages[page.id] = page;
    }
    this.tags = tagList;
    this.sortTags();
    this.initialized = true;
    return this.files;
  };

  /**
   * Removes database entries for deleted photos.
   * @param photo - The name of the photo to remove.
   */
  public removeDeleted = async (photo: string) => {
    /*
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
  };

  /**
   * Initializes the files object.
   * @param data - The files data.
   */
  public setFiles = (data: Record<string, Photo>) => {
    this.files = data;
    this.photoCount = Object.values(data).length;
  };

  /**
   * Automatically groups raw photos that already have a JPG or PNG version.
   * @param raws - The list of raw files.
   */
  public groupRaws = (raws: Photo[]) => {
    for (const raw of raws) {
      const baseName = raw.name.replace('.ORF', '').replace('.NRW', '');
      const jpg = this.files[`${baseName}.JPG`];
      const png = this.files[`${baseName}.PNG`];
      const base = this.files[raw.name];
      if (jpg && base) {
        jpg.rawFile = raw.thumbnail;
        base.hidden = true;
      } else if (png && base) {
        png.rawFile = raw.thumbnail;
        base.hidden = true;
      }
    }
  };

  /**
   * Sets a photo's date.
   * @param photo - The target photo.
   * @param date - The date to set.
   */
  public setDate = async (photo: string, date: string) => {
    const f = this.files[photo];
    if (f) {
      if (f.hasDate) {
        const oldDate = formatDate(f.date);
        if (this.dateMap[oldDate]) {
          const idx = this.dateMap[oldDate].findIndex(p => p.name === photo);
          if (idx !== -1) {
            this.dateMap[oldDate].splice(idx, 1);
          }
        }
      }
      await f.setDate(date);
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
      this.emit('updatePhoto', f);
    }
  };

  /**
   * Validates photos when a tag's requirements change.
   * @param tag - The tag that changed.
   */
  public handleTagChange = (tag: string) => {
    for (const [name, photo] of Object.entries(this.files)) {
      if (photo.tags.includes(tag)) {
        this.validateTags(name);
      }
    }
  };

  /**
   * Helper method for getting a tag's color;
   * @param tag - The tag to get.
   */
  public getTagColor = (tag: string) => {
    const at = this.advTags.find(t => t.name === tag);
    if (at) {
      return at.color;
    }
    return 'black';
  };

  /**
   * Validates tags for a photo.
   * TODO - cache the validation status so it doesn't call this function a billion times
   * @param photo - The photo to validate.
   */
  public validateTags = (photo: string) => {
    let valid = true;
    let msg = '';
    const tags = this.files[photo]?.tags;
    if (tags) {
      for (const tag of tags) {
        const a = this.advTags.find(t => t.name === tag);
        if (a) {
          if (a.prereqs.length > 0) {
            let allPrereqsMet = true;
            let missingPrereq = '';
            let i = 0;
            while (allPrereqsMet && i < a.prereqs.length) {
              const p = a.prereqs[i];
              if (p) {
                allPrereqsMet = allPrereqsMet && tags.includes(p);
                if (!tags.includes(p)) {
                  missingPrereq = `${p} (required by ${a.name})`;
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
                allCoreqsMet = allCoreqsMet && tags.includes(c);
                if (!tags.includes(c)) {
                  missingCoreq = `${c} (required by ${a.name})`;
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
            if (c && tags.includes(c)) {
              valid = false;
              msg = `Tag '${tag}' is incompatible!`;
            }
            i += 1;
          }
        }
      }
      const f = this.files[photo];
      if (f) {
        f.valid = valid;
        f.validationMsg = msg;
        this.emit('updatePhoto', f);
        this.emit('validationUpdate', photo);
      }
    }
  };

  /**
   * Checks a single photo to see if it matches the current search query.
   * @param photo - The photo to check.
   */
  // eslint-disable-next-line complexity
  public checkFilter = (photo: Photo) => {
    if (photo.hidden) {
      return false;
    }
    const terms = this.parseSearchTerms();
    let satisfiesRules = true;
    const rules = terms.filter(t => t.type === 'rule');
    for (const rule of rules) {
      switch (rule.comparison) {
        case 'is': {
          if (rule.value === 'video') {
            satisfiesRules = rule.negated
              ? satisfiesRules && !photo.video
              : satisfiesRules && photo.video;
          }
          break;
        }
        case 'of': {
          const mappedPeople = new Set(photo.people.map(p => this.people[p]?.name));
          satisfiesRules = rule.negated
            ? satisfiesRules && !mappedPeople.has(rule.value)
            : satisfiesRules && mappedPeople.has(rule.value);
          break;
        }
        case 'only': {
          const mappedPeople = photo.people.map(p => this.people[p]?.name);
          satisfiesRules = rule.negated
            ? satisfiesRules && (!mappedPeople.includes(rule.value) || mappedPeople.length > 1)
            : satisfiesRules && mappedPeople.indexOf(rule.value) === 0 && mappedPeople.length === 1;
          break;
        }
        case 'by': {
          if (photo.photographer.length > 0) {
            const name = this.people[photo.photographer]?.name;
            satisfiesRules = rule.negated
              ? satisfiesRules && name !== rule.value
              : satisfiesRules && name === rule.value;
          } else {
            if (!rule.negated) {
              satisfiesRules = false;
            }
          }
          break;
        }
        case 'at': {
          if (photo.hasLocation) {
            const name = this.places[photo.location]?.name;
            satisfiesRules = rule.negated
              ? satisfiesRules && name !== rule.value
              : satisfiesRules && name === rule.value;
          } else if (!rule.negated) {
            satisfiesRules = false;
          }

          break;
        }
        case 'has': {
          switch (rule.value) {
            case 'location': {
              satisfiesRules = rule.negated
                ? satisfiesRules && !photo.hasLocation
                : satisfiesRules && photo.hasLocation;
              break;
            }
            case 'rating': {
              satisfiesRules = rule.negated
                ? satisfiesRules && !photo.hasRating
                : satisfiesRules && photo.hasRating;
              break;
            }
            case 'photographer': {
              satisfiesRules = rule.negated
                ? satisfiesRules && photo.photographer.length === 0
                : satisfiesRules && photo.photographer.length > 0;
              break;
            }
            case 'date': {
              satisfiesRules = rule.negated
                ? satisfiesRules && !photo.hasDate
                : satisfiesRules && photo.hasDate;
              break;
            }
          }

          break;
        }
        case 'include': {
          break;
        }
        case '=': {
          switch (rule.target) {
            case 'date': {
              satisfiesRules = rule.negated
                ? satisfiesRules && formatDate(photo.date) !== rule.value
                : satisfiesRules && formatDate(photo.date) === rule.value;

              break;
            }
            case 'at': {
              if (photo.hasLocation) {
                satisfiesRules = rule.negated
                  ? satisfiesRules && photo.location !== rule.value
                  : satisfiesRules && photo.location === rule.value;
              } else if (!rule.negated) {
                satisfiesRules = false;
              }

              break;
            }
            case 'of': {
              satisfiesRules = rule.negated
                ? satisfiesRules && !photo.people.includes(rule.value)
                : satisfiesRules && photo.people.includes(rule.value);
              break;
            }
            case 'by': {
              if (photo.photographer.length > 0) {
                satisfiesRules = rule.negated
                  ? satisfiesRules && photo.photographer !== rule.value
                  : satisfiesRules && photo.photographer === rule.value;
              } else {
                if (!rule.negated) {
                  satisfiesRules = false;
                }
              }
              break;
            }
          }
          break;
        }
        case '<': {
          if (rule.target === 'date') {
            if (photo.hasDate) {
              const v = new Date(rule.value);
              satisfiesRules = rule.negated
                ? satisfiesRules && photo.date > v
                : satisfiesRules && photo.date < v;
            } else {
              satisfiesRules = false;
            }
          }
          break;
        }
        case '>': {
          if (rule.target === 'date') {
            if (photo.hasDate) {
              const v = new Date(rule.value);
              satisfiesRules = rule.negated
                ? satisfiesRules && photo.date < v
                : satisfiesRules && photo.date > v;
            } else {
              satisfiesRules = false;
            }
          }
          break;
        }
      }
    }
    const tags = terms.filter(t => t.type === 'tag');
    let hasAllTags = true;
    for (const tag of tags) {
      hasAllTags = tag.negated
        ? hasAllTags && !photo.hasTag(tag.value)
        : hasAllTags && photo.hasTag(tag.value);
    }
    return satisfiesRules && hasAllTags;
  };

  /**
   * Gets a file.
   * @param name - The name of the file.
   * @returns The file object.
   */
  public getFile = (name: string) => {
    return this.files[name];
  };

  /**
   * Sets a photo's location.
   * @param photo - The target photo.
   * @param location - The location to set.
   */
  public setLocation = async (photo: string, location: string) => {
    if (this.files[photo]?.hasLocation) {
      const oldPos = this.files[photo].location;
      if (this.places[oldPos]) {
        this.places[oldPos].count -= 1;
      }
      if (this.locationMap[oldPos]) {
        const idx = this.locationMap[oldPos].findIndex(p => p.name === photo);
        if (idx !== -1) {
          this.locationMap[oldPos].splice(idx, 1);
        }
      }
    }
    const f = this.files[photo];
    if (f) {
      await f.setLocation(location);
      if (this.places[location]) {
        this.places[location].count += 1;
      }
      if (!this.locationMap[location]) {
        this.locationMap[location] = [];
      }
      this.locationMap[location].push(f);
      this.emit('updatePhoto', f);
      this.emit('updateLocations');
    }
  };

  /**
   * Sets a photo's people.
   * @param photo - The target photo.
   * @param people - The people in the photo.
   */
  public setPeople = async (photo: string, people: string[]) => {
    const oldPeople = this.files[photo]?.people;
    if (oldPeople) {
      for (const person of oldPeople) {
        if (this.peoplePhotoMap[person]) {
          const idx = this.peoplePhotoMap[person].findIndex(p => p.name === photo);
          if (idx !== -1) {
            this.peoplePhotoMap[person].splice(idx, 1);
          }
        }
        if (this.people[person]) {
          this.people[person].count -= 1;
        }
      }
    }
    const f = this.files[photo];
    if (f) {
      await f.setPeople(people);
      for (const id of people) {
        if (this.people[id]) {
          this.people[id].count += 1;
        }
        if (!this.peoplePhotoMap[id]) {
          this.peoplePhotoMap[id] = [];
        }
        this.peoplePhotoMap[id].push(f);
      }
    }
  };

  /**
   * Creates a Place entry.
   * @param name - The name of the place.
   * @param pos - The latitude & longitude of the place.
   * @param layer - The layer the place belongs to.
   */
  public createPlace = async (name: string, pos: Position, category: PlaceType, layer: string) => {
    const id = uuid();
    const p = new Place(id, name, pos.lat, pos.lng, layer, category, '', '', '');
    p.isNewestPlace = true;
    const prev = this.places[this.newestPlace];
    if (prev) {
      prev.isNewestPlace = false;
    }
    this.newestPlace = id;
    this.places[id] = p;
    await invoke('create_place', {
      id,
      name,
      lat: pos.lat,
      lng: pos.lng,
      layer,
      category,
    });
    return p;
  };

  /**
   * Creates a Layer entry.
   * @param name - The name of the layer.
   */
  public createLayer = async (name: string) => {
    const id = uuid();
    const l = new Layer(id, name, '#ff0000');
    this.layers[id] = l;
    await invoke('create_layer', {
      id,
      name,
      color: '#ff0000',
    });
    return l;
  };

  /**
   * Gets a list of places in the given layer.
   * @param layer - The target layer.
   */
  public getPlacesByLayer = (layer: string) =>
    Object.values(this.places).filter(p => p.layer === layer);

  /**
   * Creates a shape.
   * @param type - The shape type.
   * @param points - The shape path.
   * @param layer - The layer the shape belongs to.
   * @param name - The name of the shape.
   */
  public createShape = async (type: ShapeType, points: Position[], layer: string, name: string) => {
    const id = uuid();
    const s = new Shape(id, type, JSON.stringify(points), layer, name);
    this.shapes[id] = s;
    await invoke('create_shape', {
      id,
      type,
      points: JSON.stringify(points),
      layer,
      name,
    });
    return s;
  };

  /**
   * Deletes a place.
   * @param place - The target place.
   */
  public deletePlace = async (place: string) => {
    await invoke('delete_place', { place });
    delete this.places[place];
  };

  /**
   * Deletes a shape.
   * @param shape - The target shape.
   */
  public deleteShape = async (shape: string) => {
    await invoke('delete_shape', { shape });
    delete this.shapes[shape];
  };

  /**
   * Update the calendar's focused date.
   * @param date - The date to focus on.
   */
  public setCalendarViewDate = (date: Date) => {
    this.calendarViewDate = date;
  };

  /**
   * TODO - This function originally created and updated journal entries and the journal page needs to be updated to call update functions for changed properties on existing entries
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
    const id = uuid();
    const entry = new JournalEntry(
      id,
      date,
      mood,
      text,
      activities.map(a => a.id).join(','),
      steps,
      '',
    );
    entry.activities = activities;
    this.journals[date] = entry;
    await invoke('create_journal_entry', {
      id,
      date,
      mood,
      text,
      activities: activities.map(a => a.id).join(','),
      steps,
      iv: '',
    });
    return this.journals[date];
  };

  /**
   * Sets a journal entry's text.
   * @param date - The date of the entry.
   * @param text - The entry text.
   */
  public setEntryText = async (date: string, text: string) => {
    await this.journals[date]?.setText(text, this.settings.encrypt === 'true', this.key);
    return this.journals[date];
  };

  /**
   * Creates a new activity.
   * @param name - The name of the activity.
   * @param icon - The icon for the activity.
   */
  public createActivity = async (name: string, icon: string) => {
    const id = uuid();
    const a = new Activity(id, icon, name);
    this.activities[id] = a;
    await invoke('create_activity', {
      id,
      icon,
      name,
    });
    return a;
  };

  /**
   * Adds a person.
   * @param name - The name of the person.
   * @param notes - Initial notes for the person.
   * @param category - Category color.
   */
  public addPerson = async (name: string, notes: string, category: string) => {
    const id = uuid();
    const p = new Person(id, name, '', notes, category);
    this.peopleMap[category]?.push(p);
    this.people[id] = p;
    await invoke('create_person', {
      id,
      name,
      photo: '',
      notes,
      category,
    });
    return p;
  };

  /**
   * Updates a person.
   * @param name - The name of the person.
   * @param notes - Initial notes for the person.
   * @param category - Category color.
   */
  public updatePerson = async (id: string, name: string, notes: string, category: string) => {
    const p = this.people[id];
    if (p) {
      if (name !== p.name) {
        await p.setName(name);
      }
      if (notes !== p.notes) {
        await p.setNotes(notes);
      }
      if (category !== p.category) {
        await p.setCategory(category);
      }
      return this.people[id];
    }
  };

  /**
   * Adds a person category.
   * @param name - The name of the category.
   * @param color - The color of the category.
   */
  public addPersonCategory = async (name: string, color: string) => {
    const id = uuid();
    const c = new PersonCategory(id, name, color);
    await invoke('create_person_category', { id, name, color });
    this.peopleCategories[id] = c;
    this.peopleMap[id] = [];
    return c;
  };

  /**
   * Hides a photo's thumbnail.
   * @param photo - The target photo.
   * @param value - If the thumbnail should be shown.
   */
  public setHideThumbnail = async (photo: string, value: boolean) => {
    await this.files[photo]?.setHideThumbnail(value);
    this.emit('updatePhoto', this.files[photo]);
  };

  /**
   * Sets a photo's photographer
   * @param photo - The target photo.
   * @param value - The target person.
   */
  public setPhotographer = async (photo: string, value: string) => {
    const old = this.files[photo]?.photographer;
    if (old && this.people[old]) {
      this.people[old].photographerCount -= 1;
    }
    if (old && this.photographerMap[old]) {
      const idx = this.photographerMap[old].findIndex(p => p.name === photo);
      if (idx !== -1) {
        this.photographerMap[old].splice(idx, 1);
      }
    }
    const f = this.files[photo];
    if (f) {
      await f.setPhotographer(value);
      const p = this.people[value];
      if (p) {
        p.photographerCount += 1;
      }
      if (!this.photographerMap[value]) {
        this.photographerMap[value] = [];
      }
      this.photographerMap[value].push(f);
      this.emit('updatePhoto', f);
    }
  };

  /**
   * Sets the folder structure.
   * @param structure - The folder structure.
   */
  public setFolderStructure = (structure: FolderStructure) => {
    this.folder = structure;
  };

  /**
   * Sets the view mode.
   * @param mode - The view mode.
   */
  public setViewMode = (mode: number) => {
    this.viewMode = mode;
  };

  /**
   * Sets the sort mode.
   * @param mode - The sort mode.
   * @param dir - The sort direction.
   */
  public setSortMode(mode: number, dir: number) {
    this.sort = [mode, dir];
  }

  /**
   * Encrypts all existing journal entries in the state & database.
   * @param password - The encryption password.
   */
  public encryptJournalEntries = async (password: string) => {
    if (!this.settings.encrypt) {
      await invoke('set_setting', {
        id: uuid(),
        setting: 'encrypt',
        value: 'true',
      });
      this.settings.encrypt = 'true';
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
        await entry.setText(entry.displayText, this.settings.encrypt === 'true', this.key);
        done += 1;
        this.emit('encryptionProgress', (done / total) * 100);
      }
      this.encrypted = true;
    }
  };

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
      const d = this.normalizeJournalDate(entry.date);
      const j = this.journals[d];
      if (j) {
        j.displayText = await decrypt(entry.displayText, this.key, entry.iv);
        if (save) {
          // TODO - this option isn't available in the UI so I'm not going to bother updating this code for now
          // await this.database?.update(j);
        }
      }
    }
    for (const page of Object.values(this.wikiPages)) {
      const w = this.wikiPages[page.id];
      if (w) {
        w.displayName = await decrypt(page.displayName, this.key, page.iv);
        w.displayContent = await decrypt(page.displayContent, this.key, page.iv);
        this.emit('updateWiki');
      }
    }
    this.encrypted = false;
    /*
    const r = this.settingsRecord.encrypted;
    if (save && r) {
      r.value = 'false';
      await this.database?.update(r);
    }
    */
    this.emit('decrypted');
  };

  /**
   * Adds a new camera.
   * @param name - The name of the camera.
   */
  public addCamera = async (name: string) => {
    const id = uuid();
    const c = new Camera(id, name);
    this.cameras[id] = c;
    await invoke('create_camera', {
      id,
      name,
    });
  };

  /**
   * Sets a photo's camera.
   * @param photo - The target photo.
   * @param camera - The camera to set.
   */
  public setCamera = async (photo: string, camera: string) => {
    const old = this.files[photo]?.camera;
    if (old && old.length > 0 && this.cameras[old]) {
      this.cameras[old].count -= 1;
    }
    await this.files[photo]?.setCamera(camera);
  };

  /**
   * Toggles light/dark mode.
   */
  public toggleTheme = async () => {
    this.theme = !this.theme;
    this.emit('toggleTheme');
    const id = uuid();
    const s = new Setting(id, 'theme', this.theme);
    this.settingsRecord.theme = s;
    await invoke('set_setting', {
      id,
      setting: 'theme',
      value: this.theme.toString(),
    });
  };

  /**
   * Performs a search.
   * @param query - The query terms.
   */
  public search = (...query: string[]) => {
    this.query = query;
    // TODO: Perform SELECT using non-tag fields
    let results = Object.values(this.files);
    // Filter results by tags
    results = results.filter(photo => this.checkFilter(photo));
    this.emit('search', results);
  };

  /**
   * Creates a new wiki page in the given path.
   * @param path - The path to create the page in.
   */
  public createWikiPage = async (path: string) => {
    let num = 1;
    if (path[0] === '/') {
      path = path.slice(1);
    }
    let conflicts = this.findWikiPageByName(`${path}/Untitled ${num.toString()}`) != undefined;
    while (conflicts) {
      num += 1;
      conflicts = this.findWikiPageByName(`${path}/Untitled ${num.toString()}`) != undefined;
    }
    const id = uuid();
    const page = new WikiPage(id, `${path}/Untitled ${num.toString()}`, '', '');
    this.wikiPages[id] = page;
    await invoke('create_wiki_page', {
      id,
      name: `${path}/Untitled ${num.toString()}`,
      content: '',
      iv: '',
    });
    if (this.settings.encrypt) {
      await page.setContent('', true, this.key);
    }
    this.emit('updateWiki');
  };

  /**
   * Update a wiki page's text.
   * @param path - The page to update.
   * @param content - The content to set.
   */
  public setWikiPageText = async (path: string, content: string) => {
    await this.wikiPages[path]?.setContent(content, this.settings.encrypt === 'true', this.key);
  };

  /**
   * Sets a wiki page's title.
   * @param page - The target page.
   * @param newTitle - The new title.
   */
  public setWikiPageTitle = async (page: string, newTitle: string) => {
    await this.wikiPages[page]?.setName(newTitle, this.settings.encrypt === 'true', this.key);
  };

  /**
   * Creates a new advTag entry.
   * @param name - The name of the tag.
   */
  public createTag = async (name: string) => {
    const id = uuid();
    await invoke('create_tag', {
      id,
      name,
    });
    this.advTags.push(new Tag(id, name, '', '', '', ''));
  };

  /**
   * Sets a photo's thumbnail property.
   * @param photo - The photo to set for.
   * @param thumbnail - The path to the thumbnail.
   */
  private async setThumbnail(photo: string, thumbnail: string) {
    await this.files[photo]?.setThumbnail(thumbnail);
    this.emit('updatePhoto', this.files[photo]);
  }

  /**
   * Sets & sorts the tag list.
   * @param tags - The unsorted tags.
   */
  private sortTags() {
    const tagGraph = new Graph();
    for (const tag of this.tags) {
      if (!tagGraph.get(tag)) {
        tagGraph.nodes.push(new GraphNode(tag));
      }
      const adv = this.advTags.find(t => t.name === tag);
      if (adv && adv.prereqs.length > 0) {
        for (const p of adv.prereqs) {
          if (tagGraph.get(p)) {
            const gn = tagGraph.get(p) as GraphNode;
            if (!gn.links.includes(tag)) {
              tagGraph.get(p)?.links.push(tag);
            }
          } else {
            const gn = new GraphNode(p);
            gn.links.push(tag);
            tagGraph.nodes.push(gn);
          }
        }
      }
    }
    this.tags = tagGraph.toSorted();
  }

  private normalizeJournalDate(date: string | Date) {
    if (typeof date === 'string') {
      return formatDate(new Date(date));
    }
    return formatDate(date);
  }

  private parseSearchTerms() {
    const terms: SearchTerm[] = [];
    for (let term of this.query) {
      let matched = false;
      let negated = false;
      if (term[0] === '-') {
        negated = true;
        term = term.slice(1);
      }
      if (term.includes('=')) {
        const [target, value] = term.split('=');
        if (target && value && ['date', 'at', 'of', 'by'].includes(target)) {
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
          if (['location', 'rating', 'photographer', 'date'].includes(value)) {
            terms.push({
              type: 'rule',
              comparison: 'has',
              value,
              negated,
            });
            matched = true;
          }
        } else if (target === 'include' && value && ['duplicates'].includes(value)) {
          terms.push({
            type: 'rule',
            comparison: 'include',
            value,
            negated,
          });
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
    }
    return terms;
  }

  private findWikiPageByName = (name: string) =>
    Object.values(this.wikiPages).find(p => p.name === name);
}

const f = new FileStore();
for (const key of Object.getOwnPropertyNames(Object.getPrototypeOf(f))) {
  if (key !== 'constructor') {
    f[key] =
      typeof Object.getPrototypeOf(f)[key] === 'function'
        ? (...args: any[]) => {
            console.log(key, args);
            return Object.getPrototypeOf(f)[key].call(f, ...args);
          }
        : Object.getPrototypeOf(f)[key].bind(f);
  }
}
export const fileStore = f;
