import type { Activity } from './Activity';
import { invoke } from '@tauri-apps/api/core';
import { ab2b64, b642ab, encrypt } from '@/util/encrypt';

export class JournalEntry {
  public activities: Activity[] = [];

  public incomingLinks: string[] = [];

  public outgoingLinks: string[] = [];

  public displayText = '';

  public constructor(
    private id: string,
    private _date: string,
    private _mood: number,
    private _text: string,
    private _activities: string,
    private _steps: number,
    private _iv: string,
  ) {
    this.displayText = _text;
  }

  public get activitiesStr() {
    return this._activities;
  }

  public get date() {
    return new Date(this._date);
  }

  public get mood() {
    return this._mood;
  }

  public get steps() {
    return this._steps;
  }

  public get iv() {
    return this._iv;
  }

  public async setMood(mood: number) {
    this._mood = mood;
    await invoke('set_journal_mood', {
      journal: this.id,
      mood,
    });
  }

  /**
   * Sets an unencrypted value, and encrypts it if necessary.
   * @param content - The unencrypted content.
   * @param encrypted - If the data should be encrypted in the database.
   * @param key - If encrypted, the user's encryption key.
   */
  public async setText(content: string, encrypted: boolean, key?: CryptoKey) {
    this.displayText = content;
    let finalContent = content;
    if (encrypted && key) {
      if (this.iv.length > 0) {
        finalContent = (await encrypt(content, key, b642ab(this.iv))).text;
      } else {
        // Create a new IV for a page that was not previously encrypted
        const res = await encrypt(content, key);
        finalContent = res.text;
        this.iv = ab2b64(res.iv);
        await invoke('set_journal_str', {
          page: this.id,
          property: 'iv',
          value: this.iv,
        });
      }
    }
    this._text = finalContent;
    await invoke('set_journal_str', {
      journal: this.id,
      property: 'text',
      value: finalContent,
    });
  }
}
