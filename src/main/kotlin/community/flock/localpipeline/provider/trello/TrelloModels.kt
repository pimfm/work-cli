package community.flock.localpipeline.provider.trello

import kotlinx.serialization.Serializable

@Serializable
data class TrelloCard(
    val id: String,
    val name: String,
    val desc: String? = null,
    val shortUrl: String,
    val idList: String,
    val labels: List<TrelloLabel> = emptyList(),
    val idBoard: String,
)

@Serializable
data class TrelloLabel(
    val name: String,
    val color: String? = null,
)

@Serializable
data class TrelloList(
    val id: String,
    val name: String,
)

@Serializable
data class TrelloMember(
    val id: String,
    val fullName: String,
)

@Serializable
data class TrelloBoard(
    val id: String,
    val name: String,
)
