package community.flock.localpipeline.provider.jira

import community.flock.localpipeline.model.WorkItem
import community.flock.localpipeline.provider.WorkItemProvider
import io.ktor.client.HttpClient
import io.ktor.client.call.body
import io.ktor.client.request.basicAuth
import io.ktor.client.request.get
import io.ktor.client.request.parameter
import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.jsonArray
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive

class JiraProvider(
    private val client: HttpClient,
    private val domain: String,
    private val email: String,
    private val apiToken: String,
) : WorkItemProvider {

    override val name = "Jira"

    override suspend fun fetchAssignedItems(): List<WorkItem> {
        val baseUrl = "https://$domain.atlassian.net"
        val response: JiraSearchResponse = client.get("$baseUrl/rest/api/3/search") {
            basicAuth(email, apiToken)
            parameter("jql", "assignee = currentUser() AND statusCategory != Done ORDER BY priority ASC")
            parameter("maxResults", "50")
            parameter("fields", "summary,description,status,priority,labels,project")
        }.body()

        return response.issues.map { issue ->
            WorkItem(
                id = issue.key,
                title = issue.fields.summary,
                description = extractTextFromAdf(issue.fields.description)?.take(500),
                status = issue.fields.status?.name,
                priority = issue.fields.priority?.name,
                labels = issue.fields.labels,
                source = "Jira",
                team = issue.fields.project?.name,
                url = "$baseUrl/browse/${issue.key}",
            )
        }
    }

    private fun extractTextFromAdf(element: JsonElement?): String? {
        if (element == null) return null
        return when (element) {
            is JsonPrimitive -> element.content
            is JsonObject -> {
                val type = element["type"]?.jsonPrimitive?.content
                if (type == "text") {
                    element["text"]?.jsonPrimitive?.content
                } else {
                    element["content"]?.let { extractTextFromAdf(it) }
                }
            }
            is JsonArray -> {
                element.mapNotNull { extractTextFromAdf(it) }
                    .joinToString(" ")
                    .takeIf { it.isNotBlank() }
            }
        }
    }
}
