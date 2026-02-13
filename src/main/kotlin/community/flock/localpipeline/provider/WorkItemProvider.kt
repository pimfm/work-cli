package community.flock.localpipeline.provider

import community.flock.localpipeline.model.WorkItem

interface WorkItemProvider {
    val name: String
    suspend fun fetchAssignedItems(): List<WorkItem>
}
