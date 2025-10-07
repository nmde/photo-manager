import { invoke } from '@tauri-apps/api/core';
import { ab2b64, b642ab, encrypt } from '@/util/encrypt';

export class WikiPage {
  public incomingLinks: string[] = [];

  public outgoingLinks: string[] = [];

  public displayName = '';

  public displayContent = '';

  public constructor(
    private _id: string,
    private _name: string,
    private _content: string,
    private _iv: string,
  ) {
    this.displayContent = _content;
    this.displayName = _name;
  }

  public get id() {
    return this._id;
  }

  public get name() {
    return this._name;
  }

  public get iv() {
    return this._iv;
  }

  /**
   * Sets an unencrypted name, and encrypts it if necessary.
   * @param name - The unencrypted name.
   * @param encrypted - If the data should be encrypted in the database.
   * @param key - If encrypted, the user's encryption key.
   */
  public async setName(name: string, encrypted: boolean, key?: CryptoKey) {
    this.displayName = name;
    let finalContent = name;
    if (encrypted && key) {
      if (this.iv.length > 0) {
        finalContent = (await encrypt(name, key, b642ab(this.iv))).text;
      } else {
        // Create a new IV for a page that was not previously encrypted
        const res = await encrypt(name, key);
        finalContent = res.text;
        this.iv = ab2b64(res.iv);
        await invoke('set_wiki_str', {
          page: this.id,
          property: 'iv',
          value: this.iv,
        });
      }
    }
    this._name = finalContent;
    await invoke('set_wiki_str', {
      page: this.id,
      property: 'name',
      value: finalContent,
    });
  }

  /**
   * Sets an unencrypted value, and encrypts it if necessary.
   * @param content - The unencrypted content.
   * @param encrypted - If the data should be encrypted in the database.
   * @param key - If encrypted, the user's encryption key.
   */
  public async setContent(content: string, encrypted: boolean, key?: CryptoKey) {
    this.displayContent = content;
    let finalContent = content;
    if (encrypted && key) {
      if (this.iv.length > 0) {
        finalContent = (await encrypt(content, key, b642ab(this.iv))).text;
      } else {
        // Create a new IV for a page that was not previously encrypted
        const res = await encrypt(content, key);
        finalContent = res.text;
        this.iv = ab2b64(res.iv);
        await invoke('set_wiki_str', {
          page: this.id,
          property: 'iv',
          value: this.iv,
        });
      }
    }
    this._content = finalContent;
    await invoke('set_wiki_str', {
      page: this.id,
      property: 'content',
      value: finalContent,
    });
  }
}
