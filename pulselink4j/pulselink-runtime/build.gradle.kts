plugins {
    id("pulselink.kotlin-library")
    id("pulselink.publish")
}

description = "pulselink4j Coyote V3 runtime"

dependencies {
    api(project(":pulselink-core"))
    api(project(":pulselink-protocol"))
    api(project(":pulselink-transport"))
    implementation(libs.kotlinx.coroutines.core)
    implementation(libs.slf4j.api)

    testImplementation(project(":pulselink-testing"))
}
