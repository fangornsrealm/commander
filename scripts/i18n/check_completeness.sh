#!/usr/bin/env bash
#
# check_completeness.sh
# ---------------------
# Verify that every translation key used in the Rust source via the
# `fl!("...")` macro is actually defined in each locale directory under
# `i18n/`.
#
# Layout expected (i18n-embed + fluent):
#
#   src/**.rs                 -> fl!("some-key", ...) call sites
#   i18n/<locale>/*.ftl       -> fluent resource files defining keys
#
# A locale is considered "complete" when every key referenced from the
# source code is present in one of that locale's .ftl files. Keys that are
# only present in the source language will silently fall back at runtime
# (i18n-embed behaviour), so this check flags them as gaps.
#
# Exit codes:
#   0  all locales are complete (or --allow-fallback was given)
#   1  at least one locale is missing keys (and --allow-fallback not given)
#   2  usage / environment error (no source, no i18n dir, no locales)
#
# Usage:
#   scripts/i18n/check_completeness.sh [--allow-fallback] [-h|--help]
#
# Make executable once:  chmod +x scripts/i18n/check_completeness.sh
#
# Notes / known trade-offs:
#   * Keys are extracted with a text search, so `fl!("key")` occurrences that
#     appear inside comments are counted. Keep comments free of fake fl! calls.
#   * Sorting uses LC_ALL=C so `comm` comparisons are deterministic.

set -euo pipefail

# ---------------------------------------------------------------------------
# Argument parsing
# ---------------------------------------------------------------------------
ALLOW_FALLBACK=0
for arg in "$@"; do
    case "$arg" in
        -h|--help)
            cat <<'EOF'
check_completeness.sh — verify fl!("...") keys exist in every i18n locale.

Usage:
  check_completeness.sh [OPTIONS]

Options:
  --allow-fallback   Report missing keys but do not fail the build.
  -h, --help         Show this help and exit.

Exit status:
  0  all locales complete (or --allow-fallback set)
  1  one or more locales missing keys
  2  usage or environment error
EOF
            exit 0
            ;;
        --allow-fallback)
            ALLOW_FALLBACK=1
            ;;
        *)
            echo "error: unknown option: $arg" >&2
            echo "Try 'check_completeness.sh --help' for more information." >&2
            exit 2
            ;;
    esac
done

# ---------------------------------------------------------------------------
# Locate project directories (script lives in <root>/scripts/i18n/)
# ---------------------------------------------------------------------------
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

SRC_DIR="${ROOT}/src"
I18N_DIR="${ROOT}/i18n"

if [[ ! -d "${SRC_DIR}" ]]; then
    echo "error: source directory not found: ${SRC_DIR}" >&2
    exit 2
fi
if [[ ! -d "${I18N_DIR}" ]]; then
    echo "error: i18n directory not found: ${I18N_DIR}" >&2
    exit 2
fi

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------
tmpdir="$(mktemp -d)"
trap 'rm -rf "${tmpdir}"' EXIT

# Extract every fl!("...") key used in the Rust source (the key is the first
# string literal in the macro invocation). Output: one key per line, C-sorted.
extract_used_keys() {
    grep -rhoE 'fl![ \t]*\([ \t]*"[^"]+"' --include='*.rs' "${SRC_DIR}" 2>/dev/null \
        | sed -E 's/^[^"]*"//; s/"[ \t]*$//' \
        | LC_ALL=C sort -u
}

# Extract every key defined in a locale's .ftl files. A top-level fluent entry
# starts at column 0 with an identifier followed by '='. Attributes (@foo.bar)
# and indented continuation lines are deliberately excluded.
extract_locale_keys() {
    local dir="${I18N_DIR}/$1"
    [[ -d "${dir}" ]] || return 0
    # A fluent entry starts at column 0 with an identifier followed by '='.
    # Fluent identifiers may contain hyphens and underscores (e.g. `copy-noun`),
    # so allow `[A-Za-z0-9_-]` in any position. Attributes (`@foo`) and indented
    # continuation lines are deliberately excluded by the anchor.
    grep -rhoE '^[A-Za-z0-9_-]+[ \t]*=' --include='*.ftl' "${dir}" 2>/dev/null \
        | sed -E 's/[ \t]*=$//' \
        | LC_ALL=C sort -u
    return 0
}

# ---------------------------------------------------------------------------
# Discover locales
# ---------------------------------------------------------------------------
locales=()
for entry in "${I18N_DIR}"/*/; do
    [[ -d "${entry}" ]] || continue
    locales+=("$(basename "${entry}")")
done

if [[ ${#locales[@]} -eq 0 ]]; then
    echo "error: no locale directories found under ${I18N_DIR}" >&2
    exit 2
fi

# ---------------------------------------------------------------------------
# Gather the set of used keys
# ---------------------------------------------------------------------------
used_file="${tmpdir}/used.keys"
extract_used_keys > "${used_file}"

total_used="$(wc -l < "${used_file}" | tr -d '[:space:]')"
if [[ "${total_used}" -eq 0 ]]; then
    echo "warning: no fl!(\"…\") keys found in ${SRC_DIR}; nothing to check" >&2
    exit 0
fi

echo "Found ${total_used} unique key(s) used in the source across ${#locales[@]} locale(s)."
echo

# ---------------------------------------------------------------------------
# Compare each locale against the used keys
# ---------------------------------------------------------------------------
overall=0
for locale in "${locales[@]}"; do
    defined_file="${tmpdir}/${locale}.keys"
    extract_locale_keys "${locale}" > "${defined_file}"

    # comm -23 = lines present in used_file but absent from defined_file.
    # The .keys files were produced with `LC_ALL=C sort`, so `comm` must run
    # under the same collation order or it will report "not in sorted order".
    missing="$(LC_ALL=C comm -23 "${used_file}" "${defined_file}")"
    missing_count=0
    if [[ -n "${missing}" ]]; then
        missing_count="$(wc -l <<< "${missing}" | tr -d '[:space:]')"
    fi

    if [[ "${missing_count}" -eq 0 ]]; then
        printf 'OK    %-8s all %s key(s) present\n' "${locale}" "${total_used}"
    else
        if [[ "${ALLOW_FALLBACK}" -eq 1 ]]; then
            printf 'WARN  %-8s %s/%s key(s) missing (fallback to source language)\n' \
                "${locale}" "${missing_count}" "${total_used}"
        else
            printf 'MISS  %-8s %s/%s key(s) missing\n' \
                "${locale}" "${missing_count}" "${total_used}"
            overall=1
        fi
        while IFS= read -r key; do
            [[ -n "${key}" ]] && printf '        - %s\n' "${key}"
        done <<< "${missing}"
    fi
done

echo
if [[ "${overall}" -eq 0 ]]; then
    if [[ "${ALLOW_FALLBACK}" -eq 1 ]]; then
        echo "Done (missing keys reported but allowed via --allow-fallback)."
    else
        echo "All locales are complete."
    fi
else
    echo "One or more locales are missing keys used in the source." >&2
    echo "Add the missing entries to the locale's .ftl files, or rerun with" >&2
    echo "--allow-fallback to only report them." >&2
fi

exit "${overall}"
