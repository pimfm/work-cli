package community.flock.localpipeline.provider.linear

import community.flock.localpipeline.model.WorkItem
import community.flock.localpipeline.provider.WorkItemProvider
import io.ktor.client.HttpClient
import io.ktor.client.call.body
import io.ktor.client.request.header
import io.ktor.client.request.post
import io.ktor.client.request.setBody
import io.ktor.http.ContentType
import io.ktor.http.contentType
import kotlinx.serialization.Serializable

class LinearProvider(
    private val client: HttpClient,
    private val apiKey: String,
) : WorkItemProvider {

    override val name = "Linear"

    @Serializable
    private data class GraphQLRequest(val query: String)

    override suspend fun fetchAssignedItems(): List<WorkItem> {
        val query = """
            {
              viewer {
                assignedIssues(
                  filter: { state: { type: { nin: ["completed", "canceled"] } } }
                  first: 50
                ) {
                  nodes {
                    id
                    identifier
                    title
                    description
                    priority
                    url
                    state { name }
                    team { name }
                    labels { nodes { name } }
                  }
                }
              }
            }
        """.trimIndent()

        val response: LinearGraphQLResponse = client.post("https://api.linear.app/graphql") {
            contentType(ContentType.Application.Json)
            header("Authorization", apiKey)
            setBody(GraphQLRequest(query))
        }.body()

        return response.data?.viewer?.assignedIssues?.nodes?.map { issue ->
            WorkItem(
                id = issue.identifier,
                title = issue.title,
                description = issue.description?.take(500),
                status = issue.state?.name,
                priority = priorityLabel(issue.priority),
                labels = issue.labels?.nodes?.map { it.name } ?: emptyList(),
                source = "Linear",
                team = issue.team?.name,
                url = issue.url,
            )
        } ?: emptyList()
    }

    private fun priorityLabel(priority: Int): String = when (priority) {
        1 -> "Urgent"
        2 -> "High"
        3 -> "Medium"
        4 -> "Low"
        else -> "None"
    }
}
