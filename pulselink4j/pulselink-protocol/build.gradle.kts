plugins {
    id("pulselink.kotlin-library")
    id("pulselink.serialization")
    id("pulselink.publish")
}

description = "pulselink4j sans-io protocol codecs"

dependencies {
    api(project(":pulselink-core"))
    api(libs.kotlinx.serialization.json)
}
