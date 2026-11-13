import net.ltgt.gradle.errorprone.CheckSeverity
import net.ltgt.gradle.errorprone.errorprone

plugins {
    `java-library`
    id("net.ltgt.errorprone")
    id("pulselink.test")
}

group = "top.chiloven"
version = "0.1.0"

val javaToolchainVersion = providers.gradleProperty("javaToolchainVersion")
        .map(String::toInt)
        .orElse(25)

java {
    toolchain {
        languageVersion.set(JavaLanguageVersion.of(javaToolchainVersion.get()))
    }
    withSourcesJar()
    withJavadocJar()
}

dependencies {
    api("org.jspecify:jspecify:1.0.1")
    errorprone("com.google.errorprone:error_prone_core:2.52.0")
    errorprone("com.uber.nullaway:nullaway:0.14.3")
}

tasks.withType<JavaCompile>().configureEach {
    options.release.set(21)
    options.compilerArgs.addAll(listOf("-Xlint:all", "-Werror"))
    if (javaToolchainVersion.get() <= 21) {
        options.compilerArgs.add("-XDaddTypeAnnotationsToSymbol=true")
    }
    options.errorprone {
        disableWarningsInGeneratedCode.set(true)
        check("NullAway", CheckSeverity.ERROR)
        option("NullAway:OnlyNullMarked", "true")
        option("NullAway:JSpecifyMode", "true")
    }
}

tasks.named<JavaCompile>("compileTestJava") {
    options.errorprone.disable("NullAway")
}

tasks.withType<Javadoc>().configureEach {
    isFailOnError = true
    (options as StandardJavadocDocletOptions)
            .addStringOption("Xdoclint:all,-missing", "-quiet")
}
