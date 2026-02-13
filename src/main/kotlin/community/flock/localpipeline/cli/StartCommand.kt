package community.flock.localpipeline.cli

import com.github.ajalt.clikt.command.SuspendingCliktCommand
import com.github.ajalt.clikt.core.terminal
import com.github.ajalt.mordant.rendering.TextColors.*
import com.github.ajalt.mordant.rendering.TextStyles.*
import community.flock.localpipeline.config.loadConfig
import community.flock.localpipeline.provider.ProviderRegistry
import community.flock.localpipeline.ui.runInteractiveList
import kotlinx.coroutines.async
import kotlinx.coroutines.awaitAll
import kotlinx.coroutines.coroutineScope

class StartCommand : SuspendingCliktCommand(name = "start") {

    override suspend fun run() {
        val config = loadConfig()
        val providers = ProviderRegistry.createProviders(config)

        if (providers.isEmpty()) {
            echo(yellow("No providers configured."))
            echo("Create a config file at ~/.localpipeline/config.toml with at least one provider section.")
            echo("")
            echo("Example:")
            echo(dim("""
                |[linear]
                |api_key = "lin_api_xxxx"
                |
                |[trello]
                |api_key = "your_key"
                |token = "your_token"
                |
                |[jira]
                |domain = "yourcompany"
                |email = "you@company.com"
                |api_token = "your_token"
            """.trimMargin()))
            return
        }

        echo(dim("Fetching work items from ${providers.joinToString { it.name }}..."))

        val items = coroutineScope {
            providers.map { provider ->
                async {
                    try {
                        provider.fetchAssignedItems()
                    } catch (e: Exception) {
                        echo(red("Error fetching from ${provider.name}: ${e.message}"))
                        emptyList()
                    }
                }
            }.awaitAll().flatten()
        }

        if (items.isEmpty()) {
            echo(yellow("No assigned work items found."))
            return
        }

        echo(dim("Found ${items.size} item(s). Launching interactive view...\n"))
        runInteractiveList(terminal, items)
    }
}
