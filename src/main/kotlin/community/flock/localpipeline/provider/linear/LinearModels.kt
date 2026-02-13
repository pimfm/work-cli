package community.flock.localpipeline.provider.linear

import kotlinx.serialization.Serializable

@Serializable
data class LinearGraphQLResponse(
    val data: LinearData? = null,
)

@Serializable
data class LinearData(
    val viewer: LinearViewer,
)

@Serializable
data class LinearViewer(
    val assignedIssues: LinearIssueConnection,
)

@Serializable
data class LinearIssueConnection(
    val nodes: List<LinearIssue>,
)

@Serializable
data class LinearIssue(
    val id: String,
    val identifier: String,
    val title: String,
    val description: String? = null,
    val priority: Int = 0,
    val url: String,
    val state: LinearState? = null,
    val team: LinearTeam? = null,
    val labels: LinearLabelConnection? = null,
)

@Serializable
data class LinearState(
    val name: String,
)

@Serializable
data class LinearTeam(
    val name: String,
)

@Serializable
data class LinearLabelConnection(
    val nodes: List<LinearLabel>,
)

@Serializable
data class LinearLabel(
    val name: String,
)
