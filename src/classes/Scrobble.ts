import { Entity } from './Entity';

type ScrobbleData = {
  timestamp: string;
};

export class Scrobble extends Entity<ScrobbleData> {
  public constructor(data: ScrobbleData) {
    super('Scrobble', data);
  }
}
