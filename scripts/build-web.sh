#!/usr/bin/env sh
set -eu

repository_root=$(git rev-parse --show-toplevel)
cd "$repository_root"

if [ -n "$(git status --porcelain)" ]; then
    printf '%s\n' "Refusing to build a release from a dirty worktree." >&2
    exit 1
fi

commit=${1:-$(git rev-parse HEAD)}
commit=$(git rev-parse --verify "${commit}^{commit}")
head_commit=$(git rev-parse HEAD)
if [ "$commit" != "$head_commit" ]; then
    printf '%s\n' "The requested commit must match the checked-out source." >&2
    exit 1
fi

trunk_command=${TRUNK:-trunk}
if ! command -v "$trunk_command" >/dev/null 2>&1 && [ -x "$HOME/.cargo/bin/trunk" ]; then
    trunk_command="$HOME/.cargo/bin/trunk"
fi
if ! command -v "$trunk_command" >/dev/null 2>&1; then
    printf '%s\n' "Trunk is required. Install it with: cargo install trunk --locked" >&2
    exit 1
fi

built_at=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
SNAKE_GIT_SHA="$commit" "$trunk_command" build --release

cat > dist/game-manifest.json <<EOF
{
  "repository": "https://github.com/theshoeman1224/snake",
  "commit": "$commit",
  "builtAt": "$built_at"
}
EOF

printf '%s\n' "Built dist/ from commit $commit"
