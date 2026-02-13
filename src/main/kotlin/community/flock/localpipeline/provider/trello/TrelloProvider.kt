package community.flock.localpipeline.provider.trello

import community.flock.localpipeline.model.WorkItem
import community.flock.localpipeline.provider.WorkItemProvider
import io.ktor.client.HttpClient
import io.ktor.client.call.body
import io.ktor.client.request.get
import io.ktor.client.request.parameter

class TrelloProvider(
    private val client: HttpClient,
    private val apiKey: String,
    private val token: String,
) : WorkItemProvider {

    override val name = "Trello"

    override suspend fun fetchAssignedItems(): List<WorkItem> {
        val member: TrelloMember = client.get("https://api.trello.com/1/members/me") {
            parameter("key", apiKey)
            parameter("token", token)
        }.body()

        val cards: List<TrelloCard> = client.get("https://api.trello.com/1/members/${member.id}/cards") {
            parameter("key", apiKey)
            parameter("token", token)
            parameter("fields", "id,name,desc,shortUrl,idList,labels,idBoard")
        }.body()

        if (cards.isEmpty()) return emptyList()

        // Collect unique board IDs and fetch board names + lists
        val boardIds = cards.map { it.idBoard }.distinct()
        val boardNames = mutableMapOf<String, String>()
        val listNames = mutableMapOf<String, String>()

        for (boardId in boardIds) {
            val board: TrelloBoard = client.get("https://api.trello.com/1/boards/$boardId") {
                parameter("key", apiKey)
                parameter("token", token)
                parameter("fields", "id,name")
            }.body()
            boardNames[boardId] = board.name

            val lists: List<TrelloList> = client.get("https://api.trello.com/1/boards/$boardId/lists") {
                parameter("key", apiKey)
                parameter("token", token)
                parameter("fields", "id,name")
            }.body()
            lists.forEach { listNames[it.id] = it.name }
        }

        return cards.map { card ->
            WorkItem(
                id = card.id.take(8),
                title = card.name,
                description = card.desc?.takeIf { it.isNotBlank() }?.take(500),
                status = listNames[card.idList],
                priority = null,
                labels = card.labels.mapNotNull { it.name.takeIf { n -> n.isNotBlank() } },
                source = "Trello",
                team = boardNames[card.idBoard],
                url = card.shortUrl,
            )
        }
    }
}
