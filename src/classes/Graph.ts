import { GraphNode } from './GraphNode';

export class Graph {
  public nodes: GraphNode[] = [];

  public get(key: string) {
    return this.nodes.find((n) => n.label === key);
  }

  public get leaves() {
    return this.nodes.filter((n) => n.links.length === 0);
  }

  private removeNode(key: string) {
    let target = -1;
    this.nodes.forEach((node, i) => {
      if (node.label === key) {
        target = i;
      }
      if (node.links.indexOf(key) >= 0) {
        node.links.splice(node.links.indexOf(key), 1);
      }
    });
    this.nodes.splice(target, 1);
  }

  public sort() {
    let sorted: string[] = [];
    let i = 0;
    const max = this.nodes.length;
    while (this.nodes.length > 0 && i < max) {
      const leaves = this.leaves.map((n) => n.label).sort();
      leaves.forEach((n) => {
        this.removeNode(n);
      });
      sorted = sorted.concat(leaves);
      i += 1;
    }
    if (i === max || i === max - 1) {
      console.log(`Possible loop detected!`);
    }
    if (this.nodes.length > 0) {
      sorted = sorted.concat(this.nodes.map((n) => n.label).sort());
    }
    return sorted.reverse();
  }
}
