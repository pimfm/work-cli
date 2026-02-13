package community.flock.localpipeline.model

data class WorkItem(
    val id: String,
    val title: String,
    val description: String? = null,
    val status: String? = null,
    val priority: String? = null,
    val labels: List<String> = emptyList(),
    val source: String,
    val team: String? = null,
    val url: String? = null,
)
