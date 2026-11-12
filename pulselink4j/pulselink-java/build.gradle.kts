plugins {
    id("pulselink.java-library")
    id("pulselink.publish")
}

description = "pulselink4j Java 21 extension"

dependencies {
    api(project(":pulselink-runtime"))
    api(project(":pulselink-ws"))
    api(project(":pulselink-ble"))
}
