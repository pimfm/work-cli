package community.flock.localpipeline

import com.github.ajalt.clikt.command.main
import community.flock.localpipeline.cli.LocalPipelineCommand

suspend fun main(args: Array<String>) = LocalPipelineCommand().main(args)
