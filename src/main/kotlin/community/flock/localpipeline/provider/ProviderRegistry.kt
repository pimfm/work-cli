package community.flock.localpipeline.provider

import community.flock.localpipeline.client.createHttpClient
import community.flock.localpipeline.config.AppConfig
import community.flock.localpipeline.provider.jira.JiraProvider
import community.flock.localpipeline.provider.linear.LinearProvider
import community.flock.localpipeline.provider.trello.TrelloProvider

object ProviderRegistry {

    fun createProviders(config: AppConfig): List<WorkItemProvider> {
        val client = createHttpClient()
        return buildList {
            config.linear?.let { linear ->
                add(LinearProvider(client, linear.api_key))
            }
            config.trello?.let { trello ->
                add(TrelloProvider(client, trello.api_key, trello.token))
            }
            config.jira?.let { jira ->
                add(JiraProvider(client, jira.domain, jira.email, jira.api_token))
            }
        }
    }
}
