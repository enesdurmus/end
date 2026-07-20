export type Result = {
  id: string;
  type: "app" | "file" | "clipboard" | "snippet";
  title: string;
  subtitle?: string;
  run: () => void | Promise<void>;
};
