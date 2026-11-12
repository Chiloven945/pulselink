plugins {
    id("pulselink.kotlin-library")
    id("pulselink.publish")
}

description = "pulselink4j device profiles and local sessions"

dependencies {
    api(project(":pulselink-core"))
    api(project(":pulselink-protocol"))
    api(project(":pulselink-transport"))
    api(project(":pulselink-runtime"))
    implementation(libs.kotlinx.coroutines.core)
    implementation(libs.slf4j.api)
}
