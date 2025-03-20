export class TokenGraph {
  private adjacencyList: Map<string, Set<string>>;

  constructor() {
    this.adjacencyList = new Map();
  }

  addPair(pair: [string, string]): void {
    const [currency1, currency2] = pair;

    if (!this.adjacencyList.has(currency1)) {
      this.adjacencyList.set(currency1, new Set());
    }
    if (!this.adjacencyList.has(currency2)) {
      this.adjacencyList.set(currency2, new Set());
    }

    this.adjacencyList.get(currency1)!.add(currency2);
    this.adjacencyList.get(currency2)!.add(currency1);
  }

  findAllPairs(): [string, string][] {
    const allCurrencies = Array.from(this.adjacencyList.keys());
    const pairs: Set<string> = new Set();

    for (let i = 0; i < allCurrencies.length; i+=1) {
      for (let j = i + 1; j < allCurrencies.length; j+=1) {
        const currency1 = allCurrencies[i];
        const currency2 = allCurrencies[j];

        if (this.hasPath(currency1, currency2)) {
          const pair = [currency1, currency2].sort().join('-');
          pairs.add(pair);
        }
      }
    }

    return Array.from(pairs).map(pair => pair.split('-') as [string, string]);
  }

  findPath(start: string, end: string): string[] | null {
    if (!this.adjacencyList.has(start) || !this.adjacencyList.has(end)) {
      return null;
    }

    const queue: string[][] = [[start]];
    const visited: Set<string> = new Set();

    while (queue.length > 0) {
      const path = queue.shift()!;
      const node = path[path.length - 1];

      if (node === end) {
        return path;
      }

      if (!visited.has(node)) {
        visited.add(node);
        const neighbors = this.adjacencyList.get(node)!;

        neighbors.forEach((neighbor) => {
          const newPath = [...path, neighbor];
          queue.push(newPath);
        })
      }
    }

    return null; // No path found
  }

  private hasPath(start: string, end: string): boolean {
    const visited = new Set<string>();
    const queue: string[] = [start];

    while (queue.length > 0) {
      const node = queue.shift()!;
      if (node === end) {
        return true;
      }

      if (!visited.has(node)) {
        visited.add(node);
        const neighbors = this.adjacencyList.get(node) || [];
        neighbors.forEach((neighbor) => {
          if (!visited.has(neighbor)) {
            queue.push(neighbor);
          }
        })
      }
    }

    return false;
  }
}