package community.flock.localpipeline.cli

import com.github.ajalt.clikt.command.SuspendingCliktCommand
import com.github.ajalt.clikt.core.subcommands

class LocalPipelineCommand : SuspendingCliktCommand(name = "fm") {
    init {
        subcommands(StartCommand())
    }

    override suspend fun run() = Unit
}
