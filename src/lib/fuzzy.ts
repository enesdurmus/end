// ponytail: basit subsequence skorlayıcı; yetmezse fuse.js
export function fuzzyScore(query: string, target: string): number {
  const q = query.trim().toLowerCase();
  const t = target.toLowerCase();
  if (q === "") return 1;
  let score = 0;
  let ti = 0;
  let prevMatch = -2;
  for (let qi = 0; qi < q.length; qi++) {
    const ch = q[qi];
    const found = t.indexOf(ch, ti);
    if (found === -1) return 0;
    score += found === 0 ? 10 : found === prevMatch + 1 ? 5 : 1;
    prevMatch = found;
    ti = found + 1;
  }
  return score;
}

export function fuzzyFilter<T>(query: string, items: T[], key: (t: T) => string): T[] {
  return items
    .map((item) => ({ item, s: fuzzyScore(query, key(item)) }))
    .filter((x) => x.s > 0)
    .sort((a, b) => b.s - a.s)
    .map((x) => x.item);
}
