# Releasing pulselink4j

`pulselink4j` publishes nine modules to Maven Central through the Sonatype Central Portal:

```text
pulselink-core
pulselink-pattern
pulselink-protocol
pulselink-transport
pulselink-ble
pulselink-device
pulselink-runtime
pulselink-ws
pulselink-java
```

`pulselink-testing` is a test-fixture artifact and is not published.

The build uses Gradle `maven-publish` + `signing`, with
[GradleUp nmcp](https://gradleup.com/nmcp/) aggregating all modules into one Central Portal
deployment. JReleaser is intentionally not used: the project does not need GitHub release or
changelog automation yet, and nmcp keeps publishing inside Gradle.

## Prerequisites

### JDK

The release workflow uses Temurin JDK 21. NullAway JSpecify mode requires JDK
21.0.8+ (or JDK 22+); older JDK 21 builds fail with
`-XDaddTypeAnnotationsToSymbol=true` unsupported. CI installs the latest Temurin 21.

### Central Portal namespace

The Maven group is:

```text
top.chiloven
```

The namespace must be verified in the Central Portal. If `chiloven.top` cannot be verified by DNS,
change the Maven group to `io.github.chiloven945` before the first release; published coordinates
cannot be changed afterwards.

### Central Portal user token

1. Sign in at <https://central.sonatype.com/>.
2. Open the account menu and generate a **User Token**.
3. Store the token username and password separately.

Do not use the portal login password. Do not commit tokens, paste them into issues, or put them in
`gradle.properties`.

### PGP key

Maven Central requires every artifact to be signed. Generate or reuse a key:

```bash
gpg --full-generate-key
gpg --list-secret-keys --keyid-format=long
```

Export the keys:

```bash
gpg --armor --export-secret-keys KEY_ID > private.asc
gpg --armor --export KEY_ID > public.asc
```

Upload the public key to the keyservers used by Central:

```bash
gpg --keyserver keyserver.ubuntu.com --send-keys KEY_ID
gpg --keyserver keys.openpgp.org --send-keys KEY_ID
```

If `keyserver.ubuntu.com` returns 404 after `--send-keys`, submit the armored public key through its
HKP endpoint:

```bash
curl -X POST \
  --data-urlencode "keytext@public.asc" \
  https://keyserver.ubuntu.com/pks/add
```

Verify:

```bash
curl -fsS \
  "https://keyserver.ubuntu.com/pks/lookup?op=get&search=0xKEY_ID" \
  | gpg --show-keys --with-colons \
  | awk -F: '/^fpr:/{print $10}'
```

### GitHub Environment

Create an environment named `maven-central` under **Settings → Environments**. Add required
reviewers and restrict deploys to
`master` and `pulselink4j-v*`.

Add these **environment secrets**:

| Secret                    | Value                                                    |
|---------------------------|----------------------------------------------------------|
| `CENTRAL_PORTAL_USERNAME` | Central Portal User Token username                       |
| `CENTRAL_PORTAL_PASSWORD` | Central Portal User Token password                       |
| `SIGNING_KEY`             | ASCII-armored PGP private key, including BEGIN/END lines |
| `SIGNING_PASSWORD`        | PGP private-key passphrase; omit if the key has none     |

The release workflow declares `environment: maven-central`, so only that job can read the secrets.
Pull-request workflows never receive them.

## Versioning

The root build reads:

```text
-PpulselinkVersion=0.1.0
```

The default is `0.1.0`. Release versions must not contain `SNAPSHOT`. After a successful release,
tag the commit:

```bash
git tag pulselink4j-v0.1.0
git push origin pulselink4j-v0.1.0
```

## Local preflight

Run the same checks as CI:

```bash
cd pulselink4j
./gradlew clean check apiCheck dokkaGenerateHtml \
  -PjavaToolchainVersion=21 \
  -PpulselinkVersion=0.1.0
```

Publish to the local Maven repository without touching Central:

```bash
./gradlew publishToMavenLocal \
  -PjavaToolchainVersion=21 \
  -PpulselinkVersion=0.1.0
```

Inspect the artifacts:

```bash
find ~/.m2/repository/top/chiloven -path '*0.1.0*' -name '*.pom'
find ~/.m2/repository/top/chiloven -path '*0.1.0*' -name '*-sources.jar'
find ~/.m2/repository/top/chiloven -path '*0.1.0*' -name '*-javadoc.jar'
```

To test signing locally without the CI secrets:

```bash
export SIGNING_KEY="$(cat private.asc)"
export SIGNING_PASSWORD='your-passphrase'
./gradlew publishToMavenLocal -PpulselinkVersion=0.1.0
find ~/.m2/repository/top/chiloven -name '*.asc'
```

### Consumer smoke test

Create a temporary Java project that depends on the published coordinates and compile the examples
from [`java.md`](java.md). This catches missing transitive dependencies and broken POM scopes before
uploading.

## CI

`.github/workflows/jvm-ci.yml` runs:

| OS      | JDK | Toolchain |
|---------|-----|-----------|
| Ubuntu  | 21  | 21        |
| Ubuntu  | 25  | 25        |
| Windows | 21  | 21        |
| macOS   | 21  | 21        |

All jobs run `./gradlew check`; Linux also runs `apiCheck`,
`dokkaGenerateHtml`, and `publishToMavenLocal`.

## Release

Use **Actions → JVM Release → Run workflow**:

- `version`: `0.1.0`
- `publish`: `false` for a dry run, `true` to upload

With `publish=false` the workflow only runs preflight and local publication. With `publish=true` it
runs:

```bash
./gradlew publishAggregationToCentralPortal \
  -PjavaToolchainVersion=21 \
  -PpulselinkVersion=0.1.0
```

The aggregation is configured with:

```kotlin
publishingType = "USER_MANAGED"
```

The first uploads are validated but **not automatically released**. Open **Central Portal →
Deployments**, inspect the deployment, then press **Publish** manually. After the first successful
release, `AUTOMATIC` can be enabled if desired.

## Troubleshooting

### 401 Unauthorized

- Check `CENTRAL_PORTAL_USERNAME` / `CENTRAL_PORTAL_PASSWORD` are the User Token values, not the
  portal login.
- Regenerate the token if it was exposed.
- Confirm the workflow job declares `environment: maven-central`.

### Namespace not verified

The deployment fails with a namespace error. Verify `top.chiloven` in the Central Portal, or change
the Maven group before publishing.

### Missing signature

- Confirm `SIGNING_KEY` contains the complete armored private key.
- Confirm `SIGNING_PASSWORD` matches the key.
- Confirm the public key is available on a keyserver.
- Check `~/.m2/repository/top/chiloven/.../*.asc` from a local signed run.

### Version already exists

Published versions are immutable. Bump `pulselinkVersion`; never try to overwrite an existing
Central release. If a deployment was uploaded but not published, it can still be dropped in the
Central Portal.

### POM or metadata validation failure

Run `publishToMavenLocal` and inspect the generated `.pom`, then compare with the Central
requirements: name, description, URL, license, developers, and SCM must all be present. The
publishing convention supplies these fields.

## Security

- Store tokens and private keys only in the `maven-central` environment.
- Rotate a token immediately if it appears in a screenshot, log, or commit.
- Delete local private-key exports after loading them into GitHub.
- Do not print secrets in workflow logs; GitHub masks known secret values, but transformed values
  may not be masked.
