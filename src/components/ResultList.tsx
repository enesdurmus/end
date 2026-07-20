import { Result } from "../types";
export function ResultList({ results, selected }: { results: Result[]; selected: number }) {
  return (
    <ul className="results">
      {results.map((r, i) => (
        <li key={r.id} className={i === selected ? "row selected" : "row"}>
          <span className="title">{r.title}</span>
          {r.subtitle && <span className="subtitle">{r.subtitle}</span>}
          <span className="badge">{r.type}</span>
        </li>
      ))}
    </ul>
  );
}
