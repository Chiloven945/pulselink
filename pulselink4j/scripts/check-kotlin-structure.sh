#!/usr/bin/env bash
set -euo pipefail

# Guards the Kotlin source layout rules documented in
# /home/chiloven945/.opencode/plan/pulselink4j-split-plan.md.
#
# Rules enforced:
#   1. Throwable subtypes must never be declared as object/data object.
#   2. Dumping-ground file names are forbidden.
#   3. Production Kotlin files must stay <= 400 lines.
#   4. Test Kotlin files must stay <= 250 lines.
#   5. Wildcard imports are forbidden.

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

fail=0

report() {
    printf '%s\n' "$*" >&2
    fail=1
}

# 1. Exception objects.
while IFS=: read -r file line text; do
    report "exception-object: ${file}:${line}: ${text}"
done < <(
    grep -rnE \
        '(^|[[:space:]])(data[[:space:]]+)?object[[:space:]]+[A-Za-z0-9_]+[[:space:]]*:[[:space:]]*[A-Za-z0-9_.]*(Exception|Error)\b' \
        --include='*.kt' \
        --exclude-dir=build \
        . || true
)

# 2. Forbidden dumping-ground file names.
while IFS= read -r file; do
    report "forbidden-file: ${file}"
done < <(
    find . -type f -name '*.kt' \
        -not -path '*/build/*' \
        \( -name 'Models.kt' -o -name 'Common.kt' -o -name 'Utils.kt' \
           -o -name 'Helpers.kt' -o -name 'Types.kt' \)
)

# 3/4. Line limits.
while IFS= read -r file; do
    lines=$(wc -l < "$file")
    case "$file" in
        */src/test/*)
            if (( lines > 250 )); then
                report "test-file-too-large: ${file} (${lines} > 250)"
            fi
            ;;
        */src/main/*)
            if (( lines > 400 )); then
                report "production-file-too-large: ${file} (${lines} > 400)"
            fi
            ;;
    esac
done < <(find . -type f -name '*.kt' -not -path '*/build/*')

# 5. Wildcard imports (warning until existing Kotlin modules are migrated).
while IFS=: read -r file line text; do
    printf 'warning: wildcard-import: %s:%s: %s\n' "$file" "$line" "$text" >&2
done < <(
    grep -rnE '^import[[:space:]]+[A-Za-z0-9_.]+\.\*$' \
        --include='*.kt' \
        --exclude-dir=build \
        . || true
)

if (( fail != 0 )); then
    printf '\nKotlin structure check failed.\n' >&2
    exit 1
fi

printf 'Kotlin structure check passed.\n'
