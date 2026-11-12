plugins {
    `maven-publish`
    signing
}

group = "top.chiloven"
version = "0.1.0"

publishing {
    publications {
        create<MavenPublication>("maven") {
            from(components["java"])
            pom {
                name.set(project.name)
                description.set(project.description ?: "pulselink4j module")
                url.set("https://github.com/Chiloven945/pulselink")
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
    if (providers.gradleProperty("signingKey").isPresent) {
        sign(publishing.publications["maven"])
    }
}
