import { Entity } from './Entity';

export type WikiPageData = {
  name: string;
  content: string;
  iv: string;
};

export class WikiPage extends Entity<WikiPageData> {
  public incomingLinks: string[] = [];

  public outgoingLinks: string[] = [];

  public constructor(data: WikiPageData) {
    super('WikiPage', data);
  }
}
