import { useEffect, useRef } from "react";
import { Result } from "../types";
import { describe } from "../lib/results";
import { ROW_SHORTCUTS } from "../lib/navigation";
import { AppIcon } from "./ui/AppIcon";
import { Kbd } from "./ui/Kbd";
import { Row } from "./ui/Row";

export function ResultList({ results, selected, onSelect }: { results: Result[]; selected: number; onSelect: (i: number) => void }) {
  const selRef = useRef<HTMLDivElement>(null);
  // a mouse-driven selection is already under the pointer; scrolling it would make the list jump
  const byMouse = useRef(false);
  useEffect(() => {
    if (byMouse.current) { byMouse.current = false; return; }
    selRef.current?.scrollIntoView({ block: "nearest" });
  }, [selected]);
  const mouse = (i: number, r: Result) => ({
    onMouseMove: () => { if (i !== selected) { byMouse.current = true; onSelect(i); } },
    onClick: () => { onSelect(i); r.run(); },
  });

  return (
    <ul className="scroll-thin list-none m-0 pr-1 overflow-y-auto flex-1 min-h-0">
      {results.map((r, i) => (
        <li key={r.id} >
          <Row
            rowRef={i === selected ? selRef : null}
            selected={i === selected}
            {...mouse(i, r)}
            icon={<AppIcon title={r.title} src={r.icon} className="w-9 h-9" />}
            title={r.title}
            sub={describe(r)}
            right={i === selected ? <Kbd>↵</Kbd> : i < ROW_SHORTCUTS ? <Kbd>{`⌘${i + 1}`}</Kbd> : null}
          />
        </li>
      ))}
    </ul>
  );
}
