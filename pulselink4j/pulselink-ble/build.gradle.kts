plugins {
    id("pulselink.kotlin-library")
    id("pulselink.publish")
}

description = "pulselink4j Kable BLE backend"

dependencies {
    api(project(":pulselink-transport"))
    implementation(libs.kable.core)
    implementation(libs.kotlinx.coroutines.core)
    implementation(libs.slf4j.api)
}
