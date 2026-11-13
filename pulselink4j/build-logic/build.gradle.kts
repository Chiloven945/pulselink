plugins {
    `kotlin-dsl`
}

group = "top.chiloven.pulselink.buildlogic"
version = "0.1.0"

repositories {
    gradlePluginPortal()
    mavenCentral()
}

dependencies {
    implementation("org.jetbrains.kotlin:kotlin-gradle-plugin:2.4.21")
    implementation("org.jetbrains.kotlin:kotlin-serialization:2.4.21")
    implementation("org.jetbrains.dokka:dokka-gradle-plugin:2.3.0")
    implementation("org.jetbrains.kotlinx:binary-compatibility-validator:0.18.2")
    implementation("net.ltgt.gradle:gradle-errorprone-plugin:5.2.0")
    implementation("com.gradleup.nmcp:nmcp:1.6.2")
}
