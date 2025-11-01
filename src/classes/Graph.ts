import { GraphNode } from './GraphNode';

export class Graph<T extends object> {
  public nodes: GraphNode<T>[] = [];

  public get leaves() {
    return this.nodes.filter(n => n.links.length === 0);
  }

  public add(label: string, data: T) {
    if (!this.nodes.some(n => n.label === label)) {
      this.nodes.push(new GraphNode<T>(label, data));
    }
  }

  public get(key: string) {
    return this.nodes.find(n => n.label === key);
  }

  public toSorted(fn?: (a: GraphNode<T>, b: GraphNode<T>) => number) {
    let sorted: GraphNode<T>[] = [];
    let i = 0;
    const max = this.nodes.length;
    while (this.nodes.length > 0 && i < max) {
      const leaves = this.leaves.toSorted((a, b) => a.label.localeCompare(b.label));
      for (const n of leaves) {
        this.removeNode(n.label);
      }
      sorted = sorted.concat(leaves);
      i += 1;
    }
    if (i === max || i === max - 1) {
      console.log(`Possible loop detected!`);
    }
    if (this.nodes.length > 0) {
      sorted = fn
        ? sorted.concat(this.nodes.toSorted(fn))
        : sorted.concat(this.nodes.toSorted((a, b) => a.label.localeCompare(b.label)));
    }
    return sorted.toReversed();
  }

  private removeNode(key: string) {
    let target = -1;
    for (const [i, node] of this.nodes.entries()) {
      if (node.label === key) {
        target = i;
      }
      if (node.links.includes(key)) {
        node.links.splice(node.links.indexOf(key), 1);
      }
    }
    this.nodes.splice(target, 1);
  }
}
