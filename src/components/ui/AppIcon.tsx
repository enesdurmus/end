// ponytail: hue from title so icons look distinct without real app icons
const hue = (s: string) => [...s].reduce((a, c) => a + c.charCodeAt(0), 0) % 360;

export function AppIcon({ title, src, className }: { title: string; src?: string; className: string }) {
  return src ? (
    <img className={`flex-none rounded-md object-contain ${className}`} src={src} alt="" />
  ) : (
    <span
      className={`flex-none rounded-md grid place-items-center text-xs font-semibold text-white ${className}`}
      style={{
        background: `linear-gradient(160deg, hsl(${hue(title)} 50% 50%), hsl(${hue(title)} 45% 36%))`,
        boxShadow: "inset 0 1px 0 rgba(255,255,255,0.18), 0 1px 3px rgba(0,0,0,0.3)",
      }}
    >
      {title.trim().charAt(0).toUpperCase() || "?"}
    </span>
  );
}
