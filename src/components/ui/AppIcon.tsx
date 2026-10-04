// ponytail: hue from title so icons look distinct without real app icons
const hue = (s: string) => [...s].reduce((a, c) => a + c.charCodeAt(0), 0) % 360;

export function AppIcon({ title, src, className }: { title: string; src?: string; className: string }) {
  return src ? (
    <img className={`flex-none rounded-lg object-contain ${className}`} src={src} alt="" />
  ) : (
    <span
      className={`flex-none rounded-lg grid place-items-center text-sm font-medium text-white ${className}`}
      style={{ background: `hsl(${hue(title)} 45% 42%)` }}
    >
      {title.trim().charAt(0).toUpperCase() || "?"}
    </span>
  );
}
