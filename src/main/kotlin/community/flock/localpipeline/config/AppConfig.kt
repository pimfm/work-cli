package community.flock.localpipeline.config

import com.akuleshov7.ktoml.Toml
import com.akuleshov7.ktoml.TomlInputConfig
import kotlinx.serialization.Serializable
import kotlinx.serialization.serializer
import java.nio.file.Path
import kotlin.io.path.exists
import kotlin.io.path.readText

@Serializable
data class AppConfig(
    val linear: LinearConfig? = null,
    val trello: TrelloConfig? = null,
    val jira: JiraConfig? = null,
)

@Serializable
data class LinearConfig(
    val api_key: String,
)

@Serializable
data class TrelloConfig(
    val api_key: String,
    val token: String,
)

@Serializable
data class JiraConfig(
    val domain: String,
    val email: String,
    val api_token: String,
)

fun loadConfig(): AppConfig {
    val configPath = Path.of(System.getProperty("user.home"), ".localpipeline", "config.toml")
    if (!configPath.exists()) {
        return AppConfig()
    }
    val toml = Toml(
        inputConfig = TomlInputConfig(ignoreUnknownNames = true),
    )
    return toml.decodeFromString(serializer(), configPath.readText())
}
