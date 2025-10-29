import type { Position } from '../classes/Map';
import type { Photo } from '../classes/Photo';
import { invoke } from '@tauri-apps/api/core';
import { EventEmitter } from 'ee-ts';
import { v4 as uuid } from 'uuid';
import { decrypt } from '@/util/encrypt';
import { Activity } from '../classes/Activity';
import { Graph } from '../classes/Graph';
import { GraphNode } from '../classes/GraphNode';
import { Group } from '../classes/Group';
import { JournalEntry } from '../classes/JournalEntry';
import { Layer } from '../classes/Layer';
import { PersonCategory } from '../classes/PersonCategory';
import { Setting, type SettingKey } from '../classes/Setting';
import { Shape, type ShapeType } from '../classes/Shape';
import { Tag, type TagData } from '../classes/Tag';
import { WikiPage } from '../classes/WikiPage';

export type FolderStructure = {
  dirs: string[];
  files: string[];
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

  public generatingThumbnails = false;

  public groups: Group[] = [];

  public groupNames: string[] = [];

  public initialized = false;

  public journals: Record<string, JournalEntry> = {};

  public photoCount = 0;

  public saveError = false;

  public thumbnailProgress = 0;

  public layers: Record<string, Layer> = {};

  public shapes: Record<string, Shape> = {};

  public calendarViewDate = new Date();

  public peopleCategories: Record<string, PersonCategory> = {};

  public folder: FolderStructure = {
    dirs: [],
    files: [],
  };

  public viewMode = 0;

  public sort = [0, 1];

  public settings: {
    [key in SettingKey]: number;
  } = {
    encrypt: 0,
    theme: 0,
  };

  public encrypted = false;

  public theme = false;

  public firstDate = new Date();

  public lastDate = new Date();

  public query: string[] = [];

  public wikiPages: Record<string, WikiPage> = {};

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
   * Loads photos from the database.
   */
  public loadPhotos = async (path: string) => {
    const data = await invoke<{
      deleted: string[];
      tags: TagData[];
      person_categories: {
        id: string;
        name: string;
        color: string;
      }[];
      groups: { id: string; name: string }[];
      layers: { id: string; name: string; color: string }[];
      shapes: { id: string; shape_type: ShapeType; points: string; layer: string; name: string }[];
      activities: { id: string; name: string; icon: string }[];
      settings: { id: string; setting: SettingKey; value: number }[];
      journals: {
        id: string;
        date: string;
        mood: number;
        text: string;
        activities: string;
        steps: number;
        iv: string;
      }[];
      wiki_pages: { id: string; name: string; content: string; iv: string }[];
      photo_count: number;
    }>('open_folder', { path });
    this.photoCount = data.photo_count;
    this.advTags = Tag.createTags(data.tags);
    this.tags = [];
    for (const pcat of data.person_categories.map(
      ({ id, name, color }) => new PersonCategory(id, name, color),
    )) {
      this.peopleCategories[pcat.id] = pcat;
    }
    /*
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
        */

    this.groups = data.groups.map(({ id, name }) => new Group(id, name));
    this.groupNames = this.groups.map(g => g.name);
    for (const layer of data.layers.map(({ id, name, color }) => new Layer(id, name, color))) {
      this.layers[layer.id] = layer;
    }
    for (const shape of data.shapes.map(
      ({ id, shape_type, points, layer, name }) => new Shape(id, shape_type, points, layer, name),
    )) {
      this.shapes[shape.id] = shape;
    }
    for (const activity of data.activities.map(
      ({ id, name, icon }) => new Activity(id, icon, name),
    )) {
      this.activities[activity.id] = activity;
    }
    for (const setting of data.settings.map(
      ({ id, setting, value }) => new Setting(id, setting, value),
    )) {
      this.settings[setting.setting] = setting.value;
      this.settingsRecord[setting.setting] = setting;
      if (setting.setting === 'encrypt' && typeof setting.value === 'boolean') {
        this.encrypted = setting.value;
      } else if (setting.setting === 'theme' && typeof setting.value === 'boolean') {
        this.theme = setting.value === 1;
        if (this.theme) {
          this.emit('toggleTheme');
        }
      }
    }
    for (const entry of data.journals.map(
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
    for (const page of data.wiki_pages.map(
      ({ id, name, content, iv }) => new WikiPage(id, name, content, iv),
    )) {
      this.wikiPages[page.id] = page;
    }
    this.sortTags();
    this.initialized = true;
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
      shapeType: type,
      points: JSON.stringify(points),
      layer,
      name,
    });
    return s;
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
    await this.journals[date]?.setText(text, this.settings.encrypt === 1, this.key);
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
   * Adds a person category.
   * @param name - The name of the category.
   * @param color - The color of the category.
   */
  public addPersonCategory = async (name: string, color: string) => {
    const id = uuid();
    const c = new PersonCategory(id, name, color);
    await invoke('create_person_category', { id, name, color });
    this.peopleCategories[id] = c;
    return c;
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
      this.settings.encrypt = 1;
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
        await entry.setText(entry.displayText, this.settings.encrypt === 1, this.key);
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
   * Toggles light/dark mode.
   */
  public toggleTheme = async () => {
    this.theme = !this.theme;
    this.emit('toggleTheme');
    const id = uuid();
    const s = new Setting(id, 'theme', this.theme ? 1 : 0);
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
  public search = async (query: string[]) => {
    this.query = query;
    await invoke('search_photos', { query });
    this.emit('search', await invoke('photo_grid', { start: 0 }));
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
    await this.wikiPages[path]?.setContent(content, this.settings.encrypt === 1, this.key);
  };

  /**
   * Sets a wiki page's title.
   * @param page - The target page.
   * @param newTitle - The new title.
   */
  public setWikiPageTitle = async (page: string, newTitle: string) => {
    await this.wikiPages[page]?.setName(newTitle, this.settings.encrypt === 1, this.key);
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
    this.advTags.push(new Tag(id, name, '', [], [], []));
  };

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

  private normalizeJournalDate = (date: string | Date) =>
    typeof date === 'string' ? formatDate(new Date(date)) : formatDate(date);

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
