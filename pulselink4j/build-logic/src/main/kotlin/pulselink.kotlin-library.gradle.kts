import org.jetbrains.kotlin.gradle.dsl.JvmTarget

plugins {
    id("org.jetbrains.kotlin.jvm")
    `java-library`
    id("org.jetbrains.dokka")
    id("org.jetbrains.kotlinx.binary-compatibility-validator")
    id("pulselink.test")
}

group = "top.chiloven"

val javaToolchainVersion = providers.gradleProperty("javaToolchainVersion")
        .map(String::toInt)
        .orElse(25)

java {
    toolchain {
        languageVersion.set(JavaLanguageVersion.of(javaToolchainVersion.get()))
    }
    sourceCompatibility = JavaVersion.VERSION_21
    targetCompatibility = JavaVersion.VERSION_21
    withSourcesJar()
    withJavadocJar()
}

kotlin {
    explicitApi()
    compilerOptions {
        jvmTarget.set(JvmTarget.JVM_21)
        allWarningsAsErrors.set(true)
        freeCompilerArgs.add("-jvm-default=enable")
    }
}

dependencies {
    api("org.jspecify:jspecify:1.0.1")
}

tasks.withType<Javadoc>().configureEach {
    isFailOnError = true
}
