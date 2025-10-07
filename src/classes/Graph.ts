import type { GraphNode } from './GraphNode';

export class Graph {
  public nodes: GraphNode[] = [];

  public get leaves() {
    return this.nodes.filter(n => n.links.length === 0);
  }

  public get(key: string) {
    return this.nodes.find(n => n.label === key);
  }

  public toSorted(fn?: (a: string, b: string) => number) {
    let sorted: string[] = [];
    let i = 0;
    const max = this.nodes.length;
    while (this.nodes.length > 0 && i < max) {
      const leaves = this.leaves.map(n => n.label).toSorted();
      for (const n of leaves) {
        this.removeNode(n);
      }
      sorted = sorted.concat(leaves);
      i += 1;
    }
    if (i === max || i === max - 1) {
      console.log(`Possible loop detected!`);
    }
    if (this.nodes.length > 0) {
      sorted = fn
        ? sorted.concat(this.nodes.map(n => n.label).toSorted(fn))
        : sorted.concat(this.nodes.map(n => n.label).toSorted());
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
