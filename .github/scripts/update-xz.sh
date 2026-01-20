#!/usr/bin/env bash
set -euo pipefail

root_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
submodule_path="liblzma-sys/xz"
version_header="$root_dir/${submodule_path}/src/liblzma/api/lzma/version.h"

read_version() {
  local header="$1"
  awk '
    /^#define LZMA_VERSION_MAJOR/ { major=$3 }
    /^#define LZMA_VERSION_MINOR/ { minor=$3 }
    /^#define LZMA_VERSION_PATCH/ { patch=$3 }
    END {
      if (major == "" || minor == "" || patch == "") {
        exit 1
      }
      print major, minor, patch
    }
  ' "$header"
}

version_gt() {
  local left="$1"
  local right="$2"
  local left_major left_minor left_patch right_major right_minor right_patch

  IFS=. read -r left_major left_minor left_patch <<< "$left"
  IFS=. read -r right_major right_minor right_patch <<< "$right"

  if ((left_major > right_major)); then
    return 0
  fi
  if ((left_major == right_major && left_minor > right_minor)); then
    return 0
  fi
  if ((left_major == right_major && left_minor == right_minor && left_patch > right_patch)); then
    return 0
  fi
  return 1
}

git -C "$root_dir" submodule update --init -- "$submodule_path"

if [ ! -f "$version_header" ]; then
  echo "Missing ${version_header} after submodule init." >&2
  exit 1
fi

current_parts=$(read_version "$version_header")
read -r current_major current_minor current_patch <<< "$current_parts"
current_version="${current_major}.${current_minor}.${current_patch}"

git -C "$root_dir/$submodule_path" fetch --tags origin
tag_list=$(git -C "$root_dir/$submodule_path" tag -l)

latest_version=$(
  echo "$tag_list" \
    | awk '/^v[0-9]+\.[0-9]+\.[0-9]+$/ {print substr($0,2)}' \
    | sort -t. -k1,1n -k2,2n -k3,3n \
    | tail -1
)

if [ -z "$latest_version" ]; then
  echo "Failed to resolve latest xz tag." >&2
  exit 1
fi

if ! version_gt "$latest_version" "$current_version"; then
  if [ -n "${GITHUB_OUTPUT:-}" ]; then
    echo "updated=0" >> "$GITHUB_OUTPUT"
    echo "version=$current_version" >> "$GITHUB_OUTPUT"
  fi
  echo "xz is up to date (${current_version})."
  exit 0
fi

git -C "$root_dir/$submodule_path" checkout "v${latest_version}"

if [ -n "${GITHUB_OUTPUT:-}" ]; then
  echo "updated=1" >> "$GITHUB_OUTPUT"
  echo "version=$latest_version" >> "$GITHUB_OUTPUT"
fi

echo "Updated xz from ${current_version} to ${latest_version}."
