import { z } from "zod";

export const GhIssueLabelSchema = z.object({
  name: z.string(),
});

export const GhIssueSchema = z.object({
  number: z.number(),
  title: z.string(),
  body: z.string().optional().default(""),
  state: z.string(),
  url: z.string(),
  labels: z.array(GhIssueLabelSchema),
  repository: z.object({
    name: z.string(),
    nameWithOwner: z.string(),
  }),
});

export const GhIssueListSchema = z.array(GhIssueSchema);

export type GhIssue = z.infer<typeof GhIssueSchema>;
