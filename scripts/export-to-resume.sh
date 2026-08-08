#!/usr/bin/env sh
set -eu

if [ "$#" -ne 1 ]; then
    printf '%s\n' "Usage: $0 /path/to/resume/public/games/snake" >&2
    exit 1
fi

destination=$1
repository_root=$(git rev-parse --show-toplevel)
cd "$repository_root"

./scripts/build-web.sh
mkdir -p "$destination"
cp -R dist/. "$destination/"

printf '%s\n' "Exported the playable Snake build to $destination"
printf '%s\n' "Review and commit the generated files in the resume repository."
