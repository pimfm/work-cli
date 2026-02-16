import type { WorkItem } from "../model/work-item.js";

export interface Board {
  id: string;
  name: string;
}

export interface WorkItemProvider {
  name: string;
  fetchAssignedItems(): Promise<WorkItem[]>;
  fetchBoards?(): Promise<Board[]>;
  setBoardFilter?(boardId: string): void;
  addComment?(itemId: string, comment: string): Promise<void>;
  markItemDone?(itemId: string): Promise<void>;
}
