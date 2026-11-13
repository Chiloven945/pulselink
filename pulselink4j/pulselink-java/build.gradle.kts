import org.jetbrains.kotlin.gradle.dsl.JvmTarget

plugins {
    id("pulselink.java-library")
    id("pulselink.publish")
    alias(libs.plugins.kotlin.jvm)
    alias(libs.plugins.binary.compatibility.validator)
}

description = "pulselink4j Java 21 extension"

kotlin {
    compilerOptions {
        jvmTarget.set(JvmTarget.JVM_21)
        allWarningsAsErrors.set(true)
        freeCompilerArgs.add("-jvm-default=enable")
    }
}

dependencies {
    implementation(project(":pulselink-device"))
    implementation(project(":pulselink-runtime"))
    implementation(project(":pulselink-ws"))
    implementation(project(":pulselink-ble"))
    implementation(project(":pulselink-pattern"))
    implementation(libs.kotlinx.coroutines.core)

    testImplementation(project(":pulselink-testing"))
    testImplementation(libs.ktor.server.core)
    testImplementation(libs.ktor.server.cio)
    testImplementation(libs.ktor.server.websockets)
    testImplementation(kotlin("test"))
}

apiValidation {
    ignoredClasses.add("top.chiloven.pulselink.java.internal.*")
}
