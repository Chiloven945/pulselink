plugins {
    id("pulselink.kotlin-library")
    id("pulselink.publish")
}

description = "pulselink4j local device transport SPI"

dependencies {
    api(project(":pulselink-core"))
    api(libs.kotlinx.coroutines.core)
}
