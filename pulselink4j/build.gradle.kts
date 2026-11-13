plugins {
    base
}

group = "top.chiloven"
version = "0.1.0"

allprojects {
    group = rootProject.group
    version = rootProject.version
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
