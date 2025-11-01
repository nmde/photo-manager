export class GraphNode<T> {
  public links: string[] = [];

  public constructor(public label: string, public data: T) {}
}
