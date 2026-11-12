plugins {
    id("pulselink.kotlin-library")
}

description = "pulselink4j test fixtures and fakes"

dependencies {
    api(project(":pulselink-core"))
    api(project(":pulselink-protocol"))
    api(project(":pulselink-transport"))
    implementation(libs.kotlinx.coroutines.core)
}
