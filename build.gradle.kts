plugins {
    kotlin("jvm") version "2.1.0"
    kotlin("plugin.serialization") version "2.1.0"
    application
}

group = "community.flock"
version = "1.0-SNAPSHOT"

repositories {
    mavenCentral()
}

kotlin {
    jvmToolchain(21)
}

application {
    mainClass.set("community.flock.localpipeline.MainKt")
    applicationName = "fm"
}

dependencies {
    // CLI framework (includes Mordant)
    implementation("com.github.ajalt.clikt:clikt:5.0.3")
    // Mordant coroutines for keyboard event flows
    implementation("com.github.ajalt.mordant:mordant-coroutines:3.0.2")

    // Ktor HTTP client
    implementation("io.ktor:ktor-client-okhttp:3.1.1")
    implementation("io.ktor:ktor-client-content-negotiation:3.1.1")
    implementation("io.ktor:ktor-serialization-kotlinx-json:3.1.1")

    // Serialization
    implementation("org.jetbrains.kotlinx:kotlinx-serialization-json:1.8.1")

    // Coroutines
    implementation("org.jetbrains.kotlinx:kotlinx-coroutines-core:1.10.1")

    // TOML config parsing
    implementation("com.akuleshov7:ktoml-core:0.5.2")
    implementation("com.akuleshov7:ktoml-file:0.5.2")

    testImplementation(kotlin("test"))
}

tasks.test {
    useJUnitPlatform()
}
