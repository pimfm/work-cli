package community.flock.localpipeline.provider.jira

import kotlinx.serialization.Serializable
import kotlinx.serialization.json.JsonElement

@Serializable
data class JiraSearchResponse(
    val issues: List<JiraIssue> = emptyList(),
)

@Serializable
data class JiraIssue(
    val key: String,
    val self: String,
    val fields: JiraFields,
)

@Serializable
data class JiraFields(
    val summary: String,
    val description: JsonElement? = null,
    val status: JiraStatus? = null,
    val priority: JiraPriority? = null,
    val labels: List<String> = emptyList(),
    val project: JiraProject? = null,
)

@Serializable
data class JiraStatus(
    val name: String,
)

@Serializable
data class JiraPriority(
    val name: String,
)

@Serializable
data class JiraProject(
    val key: String,
    val name: String,
)
