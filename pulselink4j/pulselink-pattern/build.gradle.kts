plugins {
    id("pulselink.kotlin-library")
    id("pulselink.publish")
}

description = "pulselink4j waveform timeline types"

dependencies {
    api(project(":pulselink-core"))
}
