pluginManagement {
    includeBuild("build-logic")
    repositories {
        gradlePluginPortal()
        mavenCentral()
    }
}

dependencyResolutionManagement {
    repositoriesMode.set(RepositoriesMode.FAIL_ON_PROJECT_REPOS)
    repositories {
        mavenCentral()
    }
}

rootProject.name = "pulselink4j"

include(
    ":pulselink-core",
    ":pulselink-pattern",
    ":pulselink-protocol",
    ":pulselink-transport",
    ":pulselink-ble",
    ":pulselink-device",
    ":pulselink-runtime",
    ":pulselink-ws",
    ":pulselink-java",
    ":pulselink-testing",
)
