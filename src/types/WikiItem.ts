export type WikiItem = {
  name: string;
  path: string;
  files: Record<string, WikiItem>;
  folders: Record<string, WikiItem>;
  id: string;
};
