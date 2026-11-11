plugins {
    `java-library`
    `maven-publish`
    kotlin("jvm") version "2.4.21"
}

group = "top.chiloven"
version = "0.1.0"

repositories {
    mavenCentral()
}

dependencies {
    implementation(kotlin("stdlib"))
}

java {
    toolchain {
        languageVersion = JavaLanguageVersion.of(21)
    }
    withSourcesJar()
    withJavadocJar()
}
