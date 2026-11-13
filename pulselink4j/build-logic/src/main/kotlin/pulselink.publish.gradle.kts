plugins {
    `maven-publish`
    signing
    id("com.gradleup.nmcp")
}

group = "top.chiloven"

val signingKey = providers.environmentVariable("SIGNING_KEY")
        .orElse(providers.gradleProperty("signingKey"))
val signingPassword = providers.environmentVariable("SIGNING_PASSWORD")
        .orElse(providers.gradleProperty("signingPassword"))

publishing {
    publications {
        create<MavenPublication>("maven") {
            from(components["java"])
            pom {
                name.set(project.name)
                description.set(providers.provider { project.description ?: "pulselink4j module" })
                url.set("https://github.com/Chiloven945/pulselink")
                inceptionYear.set("2025")
                licenses {
                    license {
                        name.set("Apache-2.0")
                        url.set("https://www.apache.org/licenses/LICENSE-2.0")
                    }
                }
                developers {
                    developer {
                        id.set("Chiloven945")
                        name.set("Chiloven945")
                    }
                }
                scm {
                    connection.set("scm:git:https://github.com/Chiloven945/pulselink.git")
                    developerConnection.set("scm:git:ssh://git@github.com/Chiloven945/pulselink.git")
                    url.set("https://github.com/Chiloven945/pulselink")
                }
            }
        }
    }
}

signing {
    if (signingKey.isPresent) {
        useInMemoryPgpKeys(signingKey.get(), signingPassword.orNull)
        sign(publishing.publications["maven"])
    }
}
