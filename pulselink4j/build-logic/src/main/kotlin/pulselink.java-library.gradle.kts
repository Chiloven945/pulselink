plugins {
    `java-library`
    id("pulselink.test")
}

group = "top.chiloven"
version = "0.1.0"

java {
    toolchain {
        languageVersion.set(JavaLanguageVersion.of(21))
    }
    withSourcesJar()
    // The Java extension has no public classes until J7. Re-enable the Javadoc
    // jar when the first public API is added.
}

dependencies {
    api("org.jspecify:jspecify:1.0.1")
}

tasks.withType<JavaCompile>().configureEach {
    options.release.set(21)
    options.compilerArgs.addAll(listOf("-Xlint:all", "-Werror"))
}

tasks.withType<Javadoc>().configureEach {
    isFailOnError = true
}
