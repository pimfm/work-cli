import type { WorkItem } from "../../model/work-item.js";
import type { WorkItemProvider, Board, CardStatus } from "../provider.js";
import type { TrelloMember, TrelloCard, TrelloBoard, TrelloList } from "./trello-types.js";

const STATUS_LIST_NAMES: Record<CardStatus, string> = {
  in_progress: "In Progress",
  in_review: "In Review",
  done: "Done",
};

export class TrelloProvider implements WorkItemProvider {
  name = "Trello";
  private boardFilter?: string;

  constructor(
    private apiKey: string,
    private token: string,
  ) {}

  setBoardFilter(boardId: string): void {
    this.boardFilter = boardId;
  }

  async fetchBoards(): Promise<Board[]> {
    const member = await this.get<TrelloMember>("/members/me");
    const boards = await this.get<TrelloBoard[]>(`/members/${member.id}/boards`, {
      fields: "id,name",
      filter: "open",
    });
    return boards.map((b) => ({ id: b.id, name: b.name }));
  }

  private params(): URLSearchParams {
    return new URLSearchParams({ key: this.apiKey, token: this.token });
  }

  private async get<T>(path: string, extra?: Record<string, string>): Promise<T> {
    const params = this.params();
    if (extra) {
      for (const [k, v] of Object.entries(extra)) params.set(k, v);
    }
    const res = await fetch(`https://api.trello.com/1${path}?${params}`);
    if (!res.ok) {
      throw new Error(`Trello API error: ${res.status} ${res.statusText}`);
    }
    return res.json();
  }

  private async put(path: string, body: Record<string, string>): Promise<void> {
    const params = this.params();
    for (const [k, v] of Object.entries(body)) params.set(k, v);
    const res = await fetch(`https://api.trello.com/1${path}?${params}`, {
      method: "PUT",
    });
    if (!res.ok) {
      throw new Error(`Trello API error: ${res.status} ${res.statusText}`);
    }
  }

  async addComment(itemId: string, comment: string): Promise<void> {
    const params = this.params();
    params.set("text", comment);
    const res = await fetch(`https://api.trello.com/1/cards/${itemId}/actions/comments?${params}`, {
      method: "POST",
    });
    if (!res.ok) {
      throw new Error(`Trello API error: ${res.status} ${res.statusText}`);
    }
  }

  async moveCard(itemId: string, status: CardStatus): Promise<void> {
    const targetListName = STATUS_LIST_NAMES[status];

    // Get card to find its board
    const card = await this.get<TrelloCard>(`/cards/${itemId}`, { fields: "idBoard" });

    // Get lists on the board
    const lists = await this.get<TrelloList[]>(`/boards/${card.idBoard}/lists`, { fields: "id,name" });

    const targetList = lists.find(
      (l) => l.name.toLowerCase() === targetListName.toLowerCase(),
    );
    if (!targetList) {
      throw new Error(`Trello list "${targetListName}" not found on board ${card.idBoard}`);
    }

    await this.put(`/cards/${itemId}`, { idList: targetList.id });
  }

  async fetchAssignedItems(): Promise<WorkItem[]> {
    const member = await this.get<TrelloMember>("/members/me");
    const allCards = await this.get<TrelloCard[]>(`/members/${member.id}/cards`, {
      fields: "id,name,desc,shortUrl,idList,labels,idBoard",
    });

    const filteredByBoard = this.boardFilter
      ? allCards.filter((c) => c.idBoard === this.boardFilter)
      : allCards;

    if (filteredByBoard.length === 0) return [];

    const boardIds = [...new Set(filteredByBoard.map((c) => c.idBoard))];
    const boardNames = new Map<string, string>();
    const listNames = new Map<string, string>();

    await Promise.all(
      boardIds.map(async (boardId) => {
        const [board, lists] = await Promise.all([
          this.get<TrelloBoard>(`/boards/${boardId}`, { fields: "id,name" }),
          this.get<TrelloList[]>(`/boards/${boardId}/lists`, { fields: "id,name" }),
        ]);
        boardNames.set(boardId, board.name);
        for (const list of lists) {
          listNames.set(list.id, list.name);
        }
      }),
    );

    const excludedLists = new Set(["done", "in review"]);
    const cards = filteredByBoard.filter((c) => {
      const listName = listNames.get(c.idList)?.toLowerCase();
      return !listName || !excludedLists.has(listName);
    });

    return cards.map((card) => ({
      id: card.id.slice(0, 8),
      title: card.name,
      description: card.desc?.trim() ? card.desc.slice(0, 500) : undefined,
      status: listNames.get(card.idList),
      priority: undefined,
      labels: card.labels.map((l) => l.name).filter((n) => n.length > 0),
      source: "Trello",
      team: boardNames.get(card.idBoard),
      url: card.shortUrl,
    }));
  }
}
