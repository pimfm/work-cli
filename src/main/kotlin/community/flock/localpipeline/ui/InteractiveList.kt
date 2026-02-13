package community.flock.localpipeline.ui

import com.github.ajalt.mordant.input.KeyboardEvent
import com.github.ajalt.mordant.input.coroutines.receiveKeyEventsFlow
import com.github.ajalt.mordant.input.isCtrlC
import com.github.ajalt.mordant.rendering.TextColors.*
import com.github.ajalt.mordant.rendering.TextStyles.*
import com.github.ajalt.mordant.terminal.Terminal
import community.flock.localpipeline.model.WorkItem
import kotlinx.coroutines.flow.takeWhile

data class ListState(
    val items: List<WorkItem>,
    val selectedIndex: Int = 0,
    val expandedIndex: Int? = null,
)

fun renderList(state: ListState, terminalHeight: Int): String {
    val items = state.items
    if (items.isEmpty()) return yellow("No items to display.")

    // Calculate visible window with scroll offset
    val maxVisible = (terminalHeight - 4).coerceAtLeast(5) // reserve lines for footer
    val scrollOffset = when {
        state.selectedIndex < maxVisible / 2 -> 0
        state.selectedIndex > items.size - maxVisible / 2 -> (items.size - maxVisible).coerceAtLeast(0)
        else -> (state.selectedIndex - maxVisible / 2).coerceAtLeast(0)
    }
    val visibleEnd = (scrollOffset + maxVisible).coerceAtMost(items.size)

    val lines = buildString {
        appendLine(bold(white("  Assigned Work Items (${items.size})")))
        appendLine()

        for (i in scrollOffset until visibleEnd) {
            val item = items[i]
            val isSelected = i == state.selectedIndex
            val isExpanded = i == state.expandedIndex

            val marker = if (isSelected) brightCyan("> ") else "  "
            val id = dim("[${item.id}]")
            val title = if (isSelected) bold(white(item.title)) else item.title
            val source = dim(item.source.padStart(8))
            val status = item.status?.let { dim(" | $it") } ?: ""

            appendLine("$marker$id $title$source$status")

            if (isExpanded) {
                appendLine(renderDetailPanel(item))
            }
        }

        if (items.size > maxVisible) {
            appendLine()
            appendLine(dim("  Showing ${scrollOffset + 1}-$visibleEnd of ${items.size}"))
        }

        appendLine()
        append(dim("  [↑/↓] navigate  [enter] expand/collapse  [q/esc] quit"))
    }

    return lines
}

private fun renderDetailPanel(item: WorkItem): String {
    val content = buildString {
        item.status?.let { appendLine("  ${bold("Status:")}   $it") }
        item.priority?.let { appendLine("  ${bold("Priority:")} $it") }
        if (item.labels.isNotEmpty()) {
            appendLine("  ${bold("Labels:")}   ${item.labels.joinToString(", ")}")
        }
        item.team?.let { appendLine("  ${bold("Team:")}     $it") }
        item.url?.let { appendLine("  ${bold("URL:")}      ${cyan(it)}") }
        item.description?.let {
            appendLine("  ${bold("Description:")}")
            val truncated = if (it.length > 300) it.take(300) + "..." else it
            for (line in truncated.lines()) {
                appendLine("    ${dim(line)}")
            }
        }
    }
    return content.trimEnd()
}

suspend fun runInteractiveList(terminal: Terminal, items: List<WorkItem>) {
    if (items.isEmpty()) {
        terminal.println(yellow("No items to display."))
        return
    }

    var state = ListState(items = items)
    val height = terminal.size.height

    // Initial render
    terminal.cursor.hide(showOnExit = true)
    printState(terminal, state, height)

    terminal.receiveKeyEventsFlow()
        .takeWhile { event ->
            val (newState, quit) = handleKeyEvent(event, state)
            if (quit) {
                false
            } else {
                if (newState != state) {
                    state = newState
                    clearAndPrint(terminal, state, height)
                }
                true
            }
        }
        .collect {} // drive the flow

    terminal.cursor.show()
    terminal.println() // clean line after exit
}

private fun handleKeyEvent(event: KeyboardEvent, state: ListState): Pair<ListState, Boolean> {
    val key = event.key
    return when {
        key == "q" || key == "Escape" || event.isCtrlC -> state to true
        key == "ArrowUp" -> {
            val newIndex = (state.selectedIndex - 1).coerceAtLeast(0)
            state.copy(selectedIndex = newIndex) to false
        }
        key == "ArrowDown" -> {
            val newIndex = (state.selectedIndex + 1).coerceAtMost(state.items.size - 1)
            state.copy(selectedIndex = newIndex) to false
        }
        key == "Enter" -> {
            val newExpanded = if (state.expandedIndex == state.selectedIndex) null else state.selectedIndex
            state.copy(expandedIndex = newExpanded) to false
        }
        else -> state to false
    }
}

private fun printState(terminal: Terminal, state: ListState, height: Int) {
    terminal.print(renderList(state, height))
}

private fun clearAndPrint(terminal: Terminal, state: ListState, height: Int) {
    // Move cursor up and clear - use ANSI escape to clear from cursor to end of screen
    terminal.rawPrint("\u001B[2J\u001B[H") // clear screen + move to home
    printState(terminal, state, height)
}
