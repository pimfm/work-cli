export interface GhIssue {
  number: number;
  title: string;
  body: string;
  state: string;
  url: string;
  labels: Array<{ name: string }>;
  repository: { nameWithOwner: string };
}
