#!/usr/bin/env bash

set -Eeuo pipefail

export LC_ALL=C
export TZ=UTC
umask 022

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
REPO_ROOT="$(cd -- "$SCRIPT_DIR/.." && pwd -P)"
DEFAULT_INVENTORY="$REPO_ROOT/docs/specs/wasm-inventory.json"
PHP_SRC_REPOSITORY="https://github.com/php/php-src.git"
EXPECTED_PROFILES=("8.2" "8.3" "8.4" "8.5")
CONFIGURE_ARGS=(
    "--prefix=/install"
    "--disable-all"
    "--enable-cli"
    "--disable-cgi"
    "--disable-phpdbg"
    "--without-pear"
)

STAGING_DIR=""

usage() {
    cat <<'EOF'
Build the four php-src revisions pinned by docs/specs/wasm-inventory.json.

Usage:
  scripts/build-pinned-php-src.sh --output-dir PATH [--jobs N]
  scripts/build-pinned-php-src.sh --verify-pins-only [--inventory PATH]

Options:
  --output-dir PATH   New directory that will receive all four builds.
  --inventory PATH    Inventory to validate and read (defaults to the repository inventory).
  --jobs N            Positive make parallelism (defaults to the detected CPU count).
  --verify-pins-only  Print profile/tag/tag-object/tag-commit TSV; do not fetch or build.
  -h, --help          Show this help.

The output directory must not already exist. Builds are staged in a neighboring
temporary directory and published only after every provenance and hash check passes.
EOF
}

die() {
    printf 'error: %s\n' "$*" >&2
    exit 1
}

require_command() {
    command -v "$1" >/dev/null 2>&1 || die "required command not found: $1"
}

cleanup() {
    if [[ -z "$STAGING_DIR" || ! -d "$STAGING_DIR" ]]; then
        return
    fi
    case "$(basename -- "$STAGING_DIR")" in
        .php-src-build.*) rm -rf -- "$STAGING_DIR" ;;
        *) printf 'error: refusing to clean unexpected staging path: %s\n' "$STAGING_DIR" >&2 ;;
    esac
}

detected_jobs() {
    local jobs=""
    if command -v getconf >/dev/null 2>&1; then
        jobs="$(getconf _NPROCESSORS_ONLN 2>/dev/null || true)"
    fi
    if [[ ! "$jobs" =~ ^[1-9][0-9]*$ ]] && command -v sysctl >/dev/null 2>&1; then
        jobs="$(sysctl -n hw.ncpu 2>/dev/null || true)"
    fi
    if [[ ! "$jobs" =~ ^[1-9][0-9]*$ ]]; then
        jobs=1
    fi
    printf '%s\n' "$jobs"
}

sha256_file() {
    local path="$1"
    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum -- "$path" | awk '{print $1}'
    elif command -v shasum >/dev/null 2>&1; then
        shasum -a 256 -- "$path" | awk '{print $1}'
    else
        die "sha256sum or shasum is required"
    fi
}

first_version_line() {
    "$@" 2>/dev/null | sed -n '1p'
}

extract_pins() {
    local inventory="$1"
    [[ -f "$inventory" ]] || die "inventory does not exist: $inventory"
    python3 - "$inventory" <<'PY'
import json
import re
import sys
from pathlib import Path

inventory_path = Path(sys.argv[1])
expected_profiles = ["8.2", "8.3", "8.4", "8.5"]

try:
    document = json.loads(inventory_path.read_text(encoding="utf-8"))
    schema = document["metadata"]["schema"]
    if schema != "elephc.wasm-inventory.v4":
        raise ValueError(
            "metadata.schema must be 'elephc.wasm-inventory.v4' for explicit "
            "tag_object/tag_commit pins"
        )
    pins = document["metadata"]["pins"]["php_src"]
    if not isinstance(pins, list):
        raise ValueError("metadata.pins.php_src must be an array")

    by_profile = {}
    for index, pin in enumerate(pins):
        if not isinstance(pin, dict):
            raise ValueError(f"metadata.pins.php_src[{index}] must be an object")
        profile = pin.get("profile")
        tag = pin.get("tag")
        tag_object = pin.get("tag_object")
        tag_commit = pin.get("tag_commit")
        if profile in by_profile:
            raise ValueError(f"duplicate php-src profile: {profile!r}")
        if profile not in expected_profiles:
            raise ValueError(f"unexpected php-src profile: {profile!r}")
        if not isinstance(tag, str) or not re.fullmatch(
            rf"php-{re.escape(profile)}\.[0-9]+", tag
        ):
            raise ValueError(f"profile {profile}: invalid exact php-src tag {tag!r}")
        if not isinstance(tag_object, str) or not re.fullmatch(
            r"[0-9a-f]{40}", tag_object
        ):
            raise ValueError(
                f"profile {profile}: tag_object must be 40 lowercase hex characters"
            )
        if not isinstance(tag_commit, str) or not re.fullmatch(
            r"[0-9a-f]{40}", tag_commit
        ):
            raise ValueError(
                f"profile {profile}: tag_commit must be 40 lowercase hex characters"
            )
        if tag_object == tag_commit:
            raise ValueError(
                f"profile {profile}: expected an annotated tag object distinct from tag_commit"
            )
        by_profile[profile] = (tag, tag_object, tag_commit)

    actual_profiles = sorted(by_profile)
    if actual_profiles != expected_profiles:
        raise ValueError(
            "expected exactly php-src profiles "
            + ", ".join(expected_profiles)
            + "; got "
            + (", ".join(actual_profiles) if actual_profiles else "none")
        )

    tag_objects = [tag_object for _, tag_object, _ in by_profile.values()]
    if len(tag_objects) != len(set(tag_objects)):
        raise ValueError("php-src tag objects must be unique across profiles")
    tag_commits = [tag_commit for _, _, tag_commit in by_profile.values()]
    if len(tag_commits) != len(set(tag_commits)):
        raise ValueError("php-src tag commits must be unique across profiles")

    for profile in expected_profiles:
        tag, tag_object, tag_commit = by_profile[profile]
        print(f"{profile}\t{tag}\t{tag_object}\t{tag_commit}")
except (OSError, UnicodeError, json.JSONDecodeError, KeyError, TypeError, ValueError) as error:
    print(f"error: invalid php-src pin inventory: {error}", file=sys.stderr)
    raise SystemExit(2)
PY
}

verify_checkout() {
    local checkout="$1"
    local tag="$2"
    local expected_tag_object="$3"
    local expected_tag_commit="$4"
    local tag_object=""
    local tag_commit=""
    local head=""
    local status=""

    [[ -d "$checkout/.git" ]] || die "not a standalone Git checkout: $checkout"
    tag_object="$(git -C "$checkout" rev-parse --verify "refs/tags/$tag" 2>/dev/null)" \
        || die "cannot resolve tag object for $tag"
    tag_commit="$(git -C "$checkout" rev-parse --verify "${tag}^{commit}" 2>/dev/null)" \
        || die "cannot resolve ${tag}^{commit}"
    head="$(git -C "$checkout" rev-parse --verify HEAD 2>/dev/null)" \
        || die "cannot resolve checkout HEAD"
    [[ "$tag_object" == "$expected_tag_object" ]] \
        || die "tag $tag object is $tag_object, expected inventory object $expected_tag_object"
    [[ "$tag_commit" == "$expected_tag_commit" ]] \
        || die "tag $tag peels to $tag_commit, expected inventory commit $expected_tag_commit"
    [[ "$head" == "$expected_tag_commit" ]] \
        || die "checkout HEAD is $head, expected peeled tag commit $expected_tag_commit"
    if git -C "$checkout" symbolic-ref -q HEAD >/dev/null 2>&1; then
        die "checkout HEAD is attached; a detached checkout is required"
    fi
    status="$(git -C "$checkout" status --porcelain=v1 --untracked-files=all)"
    [[ -z "$status" ]] || die "checkout is not clean: $checkout"

    printf '%s\t%s\t%s\n' "$tag_object" "$tag_commit" "$head"
}

write_hash_line() {
    local base="$1"
    local relative="$2"
    local manifest="$3"
    [[ "$relative" != /* && "$relative" != *".."* ]] \
        || die "unsafe relative hash path: $relative"
    [[ -f "$base/$relative" ]] || die "cannot hash missing file: $base/$relative"
    printf '%s  %s\n' "$(sha256_file "$base/$relative")" "$relative" >>"$manifest"
}

verify_hash_manifest() {
    local base="$1"
    local manifest="$2"
    local expected=""
    local relative=""
    local actual=""

    [[ -s "$manifest" ]] || die "hash manifest is empty: $manifest"
    while IFS=' ' read -r expected relative; do
        relative="${relative# }"
        [[ "$expected" =~ ^[0-9a-f]{64}$ ]] || die "invalid SHA-256 in $manifest"
        [[ -n "$relative" && "$relative" != /* && "$relative" != *".."* ]] \
            || die "unsafe path in $manifest: $relative"
        [[ -f "$base/$relative" ]] || die "hashed file is missing: $base/$relative"
        actual="$(sha256_file "$base/$relative")"
        [[ "$actual" == "$expected" ]] \
            || die "SHA-256 mismatch for $base/$relative"
    done <"$manifest"
}

write_profile_provenance() {
    local destination="$1"
    local profile="$2"
    local tag="$3"
    local inventory_tag_object="$4"
    local inventory_tag_commit="$5"
    local tag_object="$6"
    local tag_commit="$7"
    local head="$8"
    local inventory_sha="$9"
    local php_version="${10}"
    local source_date_epoch="${11}"
    local php_sha="${12}"
    local php_info_sha="${13}"
    local git_version="${14}"
    local autoconf_version="${15}"
    local bison_version="${16}"
    local re2c_version="${17}"
    local make_version="${18}"
    local cc_version="${19}"
    shift 19

    python3 - "$destination" "$profile" "$tag" "$inventory_tag_object" \
        "$inventory_tag_commit" "$tag_object" "$tag_commit" "$head" \
        "$inventory_sha" "$php_version" "$source_date_epoch" "$php_sha" "$php_info_sha" \
        "$git_version" "$autoconf_version" "$bison_version" "$re2c_version" \
        "$make_version" "$cc_version" "$PHP_SRC_REPOSITORY" "$@" <<'PY'
import json
import re
import sys
from pathlib import Path

(
    destination,
    profile,
    tag,
    inventory_tag_object,
    inventory_tag_commit,
    tag_object,
    tag_commit,
    head,
    inventory_sha,
    php_version,
    source_date_epoch,
    php_sha,
    php_info_sha,
    git_version,
    autoconf_version,
    bison_version,
    re2c_version,
    make_version,
    cc_version,
    repository,
    *configure_args,
) = sys.argv[1:]

if inventory_tag_object != tag_object:
    raise SystemExit("inventory tag object does not match fetched tag object")
if inventory_tag_commit != tag_commit or tag_commit != head:
    raise SystemExit("inventory tag commit, fetched tag commit, and HEAD do not match")
if not re.fullmatch(r"[0-9a-f]{64}", inventory_sha):
    raise SystemExit("inventory SHA-256 is malformed")
if not re.fullmatch(r"[0-9a-f]{64}", php_sha):
    raise SystemExit("PHP binary SHA-256 is malformed")
if not re.fullmatch(r"[0-9a-f]{64}", php_info_sha):
    raise SystemExit("php-info SHA-256 is malformed")
if not all(
    [git_version, autoconf_version, bison_version, re2c_version, make_version, cc_version]
):
    raise SystemExit("one or more build tool versions are empty")

document = {
    "schema": "elephc.pinned-php-src-build.v1",
    "profile": profile,
    "repository": repository,
    "inventory_sha256": inventory_sha,
    "source": {
        "tag": tag,
        "inventory_tag_object": inventory_tag_object,
        "inventory_tag_commit": inventory_tag_commit,
        "tag_object": tag_object,
        "tag_commit": tag_commit,
        "head": head,
        "detached": True,
        "dirty": False,
        "materialization": "git archive of verified HEAD",
        "source_date_epoch": int(source_date_epoch),
    },
    "build": {
        "configure_args": configure_args,
        "ini_mode": "-n (no php.ini)",
        "tools": {
            "git": git_version,
            "autoconf": autoconf_version,
            "bison": bison_version,
            "re2c": re2c_version,
            "make": make_version,
            "cc": cc_version,
        },
    },
    "artifact": {
        "php_binary": "install/bin/php",
        "php_version": php_version,
        "php_sha256": php_sha,
        "php_info": "php-info.txt",
        "php_info_sha256": php_info_sha,
    },
}
Path(destination).write_text(
    json.dumps(document, indent=2, sort_keys=True) + "\n",
    encoding="utf-8",
)
PY
}

build_profile() {
    local work_root="$1"
    local result_root="$2"
    local profile="$3"
    local tag="$4"
    local inventory_tag_object="$5"
    local inventory_tag_commit="$6"
    local inventory_sha="$7"
    local jobs="$8"
    local checkout="$work_root/checkouts/$profile"
    local source="$work_root/sources/$profile"
    local build="$work_root/build/$profile"
    local profile_root="$result_root/$profile"
    local expected_version="${tag#php-}"
    local verification=""
    local tag_object=""
    local tag_commit=""
    local head=""
    local source_date_epoch=""
    local php_binary=""
    local php_version=""
    local php_sha=""
    local php_info_sha=""
    local hash_manifest=""

    printf '==> PHP %s: fetching pinned tag object %s\n' "$profile" "$tag"
    mkdir -p -- "$checkout" "$source" "$build" "$profile_root"
    git -C "$checkout" init --quiet
    git -C "$checkout" remote add origin "$PHP_SRC_REPOSITORY"
    GIT_TERMINAL_PROMPT=0 git -C "$checkout" fetch --quiet --no-tags --depth=1 origin \
        "refs/tags/$tag:refs/tags/$tag"
    tag_object="$(git -C "$checkout" rev-parse --verify "refs/tags/$tag")"
    [[ "$tag_object" == "$inventory_tag_object" ]] \
        || die "tag $tag object is $tag_object, expected inventory object $inventory_tag_object"
    tag_commit="$(git -C "$checkout" rev-parse --verify "${tag}^{commit}")"
    [[ "$tag_commit" == "$inventory_tag_commit" ]] \
        || die "tag $tag peels to $tag_commit, expected inventory commit $inventory_tag_commit"
    GIT_TERMINAL_PROMPT=0 git -C "$checkout" fetch --quiet --no-tags --depth=1 origin \
        "$inventory_tag_commit"
    git -C "$checkout" checkout --quiet --detach "$inventory_tag_commit"

    verification="$(
        verify_checkout \
            "$checkout" "$tag" "$inventory_tag_object" "$inventory_tag_commit"
    )"
    IFS=$'\t' read -r tag_object tag_commit head <<<"$verification"
    [[ -n "$tag_object" && -n "$tag_commit" && -n "$head" ]] \
        || die "incomplete checkout verification for $profile"
    source_date_epoch="$(git -C "$checkout" show -s --format=%ct HEAD)"
    [[ "$source_date_epoch" =~ ^[0-9]+$ ]] \
        || die "invalid source timestamp for $profile: $source_date_epoch"

    git -C "$checkout" archive --format=tar "$head" | tar -xf - -C "$source"
    [[ -x "$source/buildconf" ]] || die "php-src $profile has no executable buildconf"

    printf '==> PHP %s: buildconf and minimal CLI configure\n' "$profile"
    (
        cd -- "$source"
        SOURCE_DATE_EPOCH="$source_date_epoch" ./buildconf --force
    )
    [[ -x "$source/configure" ]] || die "buildconf did not create configure for $profile"
    (
        cd -- "$build"
        SOURCE_DATE_EPOCH="$source_date_epoch" "$source/configure" "${CONFIGURE_ARGS[@]}"
    )

    printf '==> PHP %s: make -j%s and transactional install\n' "$profile" "$jobs"
    SOURCE_DATE_EPOCH="$source_date_epoch" make -C "$build" -j"$jobs"
    SOURCE_DATE_EPOCH="$source_date_epoch" make -C "$build" \
        INSTALL_ROOT="$profile_root" install

    php_binary="$profile_root/install/bin/php"
    [[ -x "$php_binary" ]] || die "installed PHP CLI is missing for $profile"
    php_version="$("$php_binary" -n -r 'fwrite(STDOUT, PHP_VERSION);')"
    [[ "$php_version" == "$expected_version" ]] \
        || die "built PHP version is $php_version, expected $expected_version"
    "$php_binary" -n -i >"$profile_root/php-info.txt"
    [[ -s "$profile_root/php-info.txt" ]] || die "php-info.txt is empty for $profile"

    verification="$(
        verify_checkout \
            "$checkout" "$tag" "$inventory_tag_object" "$inventory_tag_commit"
    )"
    IFS=$'\t' read -r tag_object tag_commit head <<<"$verification"
    [[ "$tag_object" == "$inventory_tag_object" \
        && "$tag_commit" == "$inventory_tag_commit" \
        && "$tag_commit" == "$head" ]] \
        || die "checkout provenance changed while building $profile"

    php_sha="$(sha256_file "$php_binary")"
    php_info_sha="$(sha256_file "$profile_root/php-info.txt")"
    write_profile_provenance \
        "$profile_root/provenance.json" \
        "$profile" "$tag" "$inventory_tag_object" "$inventory_tag_commit" \
        "$tag_object" "$tag_commit" "$head" "$inventory_sha" \
        "$php_version" "$source_date_epoch" "$php_sha" "$php_info_sha" \
        "$(first_version_line git --version)" \
        "$(first_version_line autoconf --version)" \
        "$(first_version_line bison --version)" \
        "$(first_version_line re2c --version)" \
        "$(first_version_line make --version)" \
        "$(first_version_line cc --version)" \
        "${CONFIGURE_ARGS[@]}"

    hash_manifest="$profile_root/hashes.sha256"
    : >"$hash_manifest"
    write_hash_line "$profile_root" "install/bin/php" "$hash_manifest"
    write_hash_line "$profile_root" "php-info.txt" "$hash_manifest"
    write_hash_line "$profile_root" "provenance.json" "$hash_manifest"
    verify_hash_manifest "$profile_root" "$hash_manifest"
}

write_root_provenance() {
    local result_root="$1"
    local inventory_sha="$2"
    python3 - "$result_root" "$inventory_sha" "$PHP_SRC_REPOSITORY" <<'PY'
import json
import sys
from pathlib import Path

root = Path(sys.argv[1])
inventory_sha = sys.argv[2]
repository = sys.argv[3]
expected_profiles = ["8.2", "8.3", "8.4", "8.5"]
profiles = []
for profile in expected_profiles:
    path = root / profile / "provenance.json"
    document = json.loads(path.read_text(encoding="utf-8"))
    if document.get("profile") != profile:
        raise SystemExit(f"profile provenance mismatch in {path}")
    profiles.append(
        {
            "profile": profile,
            "tag": document["source"]["tag"],
            "tag_object": document["source"]["tag_object"],
            "tag_commit": document["source"]["tag_commit"],
            "php_version": document["artifact"]["php_version"],
            "php_sha256": document["artifact"]["php_sha256"],
            "provenance": f"{profile}/provenance.json",
            "hashes": f"{profile}/hashes.sha256",
        }
    )

document = {
    "schema": "elephc.pinned-php-src-build-set.v1",
    "repository": repository,
    "inventory_sha256": inventory_sha,
    "profiles": profiles,
}
(root / "provenance.json").write_text(
    json.dumps(document, indent=2, sort_keys=True) + "\n",
    encoding="utf-8",
)
PY
}

main() {
    local inventory="$DEFAULT_INVENTORY"
    local output_dir=""
    local jobs="$(detected_jobs)"
    local verify_pins_only=0
    local pins=""
    local parent=""
    local name=""
    local parent_abs=""
    local output_abs=""
    local work_root=""
    local result_root=""
    local inventory_sha=""
    local profile=""
    local tag=""
    local tag_object=""
    local tag_commit=""
    local profile_count=0
    local root_hashes=""

    while (($#)); do
        case "$1" in
            --output-dir)
                (($# >= 2)) || die "--output-dir requires a value"
                output_dir="$2"
                shift 2
                ;;
            --inventory)
                (($# >= 2)) || die "--inventory requires a value"
                inventory="$2"
                shift 2
                ;;
            --jobs)
                (($# >= 2)) || die "--jobs requires a value"
                jobs="$2"
                shift 2
                ;;
            --verify-pins-only)
                verify_pins_only=1
                shift
                ;;
            -h|--help)
                usage
                return 0
                ;;
            *)
                die "unknown argument: $1"
                ;;
        esac
    done

    require_command python3
    pins="$(extract_pins "$inventory")"
    if ((verify_pins_only)); then
        [[ -z "$output_dir" ]] || die "--output-dir is incompatible with --verify-pins-only"
        printf '%s\n' "$pins"
        return 0
    fi

    [[ -n "$output_dir" ]] || die "--output-dir is required"
    [[ "$jobs" =~ ^[1-9][0-9]*$ ]] || die "--jobs must be a positive integer"
    for command in git tar autoconf bison re2c make cc sed awk; do
        require_command "$command"
    done
    if ! command -v sha256sum >/dev/null 2>&1 \
        && ! command -v shasum >/dev/null 2>&1; then
        die "sha256sum or shasum is required"
    fi

    [[ "$output_dir" != "/" ]] || die "refusing to use / as the output directory"
    [[ ! -e "$output_dir" && ! -L "$output_dir" ]] \
        || die "output directory already exists: $output_dir"
    parent="$(dirname -- "$output_dir")"
    name="$(basename -- "$output_dir")"
    [[ -n "$name" && "$name" != "." && "$name" != ".." ]] \
        || die "invalid output directory: $output_dir"
    mkdir -p -- "$parent"
    parent_abs="$(cd -- "$parent" && pwd -P)"
    output_abs="$parent_abs/$name"
    [[ ! -e "$output_abs" && ! -L "$output_abs" ]] \
        || die "output directory already exists: $output_abs"

    STAGING_DIR="$(mktemp -d "$parent_abs/.php-src-build.XXXXXX")"
    trap cleanup EXIT
    trap 'exit 130' HUP INT TERM
    work_root="$STAGING_DIR/work"
    result_root="$STAGING_DIR/result"
    mkdir -p -- "$work_root" "$result_root"

    inventory_sha="$(sha256_file "$inventory")"
    while IFS=$'\t' read -r profile tag tag_object tag_commit; do
        [[ -n "$profile" && -n "$tag" && -n "$tag_object" && -n "$tag_commit" ]] \
            || die "incomplete canonical pin record"
        build_profile \
            "$work_root" "$result_root" "$profile" "$tag" "$tag_object" \
            "$tag_commit" "$inventory_sha" "$jobs"
        profile_count=$((profile_count + 1))
    done <<<"$pins"
    [[ "$profile_count" -eq "${#EXPECTED_PROFILES[@]}" ]] \
        || die "built $profile_count profiles, expected ${#EXPECTED_PROFILES[@]}"

    write_root_provenance "$result_root" "$inventory_sha"
    root_hashes="$result_root/hashes.sha256"
    : >"$root_hashes"
    write_hash_line "$result_root" "provenance.json" "$root_hashes"
    for profile in "${EXPECTED_PROFILES[@]}"; do
        write_hash_line "$result_root" "$profile/install/bin/php" "$root_hashes"
        write_hash_line "$result_root" "$profile/php-info.txt" "$root_hashes"
        write_hash_line "$result_root" "$profile/provenance.json" "$root_hashes"
        write_hash_line "$result_root" "$profile/hashes.sha256" "$root_hashes"
    done
    verify_hash_manifest "$result_root" "$root_hashes"

    mv -- "$result_root" "$output_abs"
    printf 'Built and verified pinned php-src CLIs in %s\n' "$output_abs"
}

if [[ "${BASH_SOURCE[0]}" == "$0" ]]; then
    main "$@"
fi
