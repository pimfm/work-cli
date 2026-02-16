import type { CardStatus, WorkItemProvider } from "../providers/provider.js";

export async function moveCard(
  providers: WorkItemProvider[],
  itemId: string,
  status: CardStatus,
): Promise<void> {
  for (const provider of providers) {
    if (!provider.moveCard) continue;
    try {
      await provider.moveCard(itemId, status);
      return;
    } catch {
      // Provider didn't match or API failed — try next
    }
  }
}
