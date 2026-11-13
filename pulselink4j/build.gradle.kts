plugins {
    base
    alias(libs.plugins.nmcp.aggregation)
}

group = "top.chiloven"
version = providers.gradleProperty("pulselinkVersion").orElse("0.1.0").get()

allprojects {
    group = rootProject.group
    version = rootProject.version
}

val centralPortalUsername = providers.environmentVariable("CENTRAL_PORTAL_USERNAME")
val centralPortalPassword = providers.environmentVariable("CENTRAL_PORTAL_PASSWORD")

nmcpAggregation {
    centralPortal {
        if (centralPortalUsername.isPresent && centralPortalPassword.isPresent) {
            username = centralPortalUsername.get()
            password = centralPortalPassword.get()
        }
        publishingType = "USER_MANAGED"
        publicationName = "pulselink4j:$version"
        validationTimeout = java.time.Duration.ofMinutes(30)
    }
}

dependencies {
    nmcpAggregation(project(":pulselink-core"))
    nmcpAggregation(project(":pulselink-pattern"))
    nmcpAggregation(project(":pulselink-protocol"))
    nmcpAggregation(project(":pulselink-transport"))
    nmcpAggregation(project(":pulselink-ble"))
    nmcpAggregation(project(":pulselink-device"))
    nmcpAggregation(project(":pulselink-runtime"))
    nmcpAggregation(project(":pulselink-ws"))
    nmcpAggregation(project(":pulselink-java"))
}

val checkKotlinStructure by tasks.registering(Exec::class) {
    group = "verification"
    description = "Checks Kotlin file structure rules."
    commandLine(
        "bash",
        layout.projectDirectory.file("scripts/check-kotlin-structure.sh")
                .asFile
                .absolutePath,
    )
}

tasks.named("check") {
    dependsOn(checkKotlinStructure)
}
