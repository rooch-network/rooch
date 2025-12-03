# Rooch Move Framework Release

This directory contains the release notes for each framework release.

## Release Notes

- [Rooch Move Framework v25](./25/README.md)
- [Rooch Move Framework v24](./24/README.md)
- [Rooch Move Framework v23](./23/README.md)
- [Rooch Move Framework v22](./22/README.md)
- [Rooch Move Framework v21](./21/README.md)

## How to release

```bash
cargo run --package framework-release --bin framework-release -- -v 23
```

Note: change the version number in the command above to the new version you want to release.

## How to generate release notes

Follow these steps to produce release notes for a new framework release. Do not rely on git tags for framework versions; use directory-introducing commits as boundaries.

1) Identify boundary commits

- Find the commit that introduced the previous version directory (vN-1):

```bash
git log --oneline --decorate --date=short --max-count=1 --diff-filter=A --name-only -- frameworks/framework-release/released/<N-1>
```

- Find the commit that introduced the current version directory (vN):

```bash
git log --oneline --decorate --date=short --max-count=1 --diff-filter=A --name-only -- frameworks/framework-release/released/<N>
```

Take note of the two SHAs: PREV=<sha_of_vN-1>, CURR=<sha_of_vN>.

2) List all commits affecting `frameworks/` between the two commits

```bash
git log --oneline --decorate --date=short PREV..CURR -- frameworks/
```

3) Inspect each commit to understand the changes

```bash
# For each <sha> from the list above
git show --name-only --pretty=format:%H%n%s%n%b <sha> -- frameworks/
```

4) Draft the release notes

- Create or update `frameworks/framework-release/released/<N>/README.md` with the header:

```markdown
# Rooch Move Framework v<N>
```

- Summarize changes as concise bullet points grouped by submodule when helpful (e.g., `[rooch-framework]`, `[moveos_std]`, `[bitcoin-move]`).
- Prefer actionable descriptions (what was added/changed/fixed) and reference PR/issue numbers if present in the subject (e.g., `(#1234)`).
- Keep all content in English and ASCII only.

5) Notes and tips

- The range PREV..CURR may include commits that touched previous release artifacts (e.g., `frameworks/framework-release/released/<N-1>`). Include such items only if they represent meaningful behavior changes (e.g., migrations) rather than artifact refreshes.

1) Example (replace with actual SHAs)

```bash
# Identify boundaries
git log --oneline --decorate --date=short --max-count=1 --diff-filter=A --name-only -- frameworks/framework-release/released/22
git log --oneline --decorate --date=short --max-count=1 --diff-filter=A --name-only -- frameworks/framework-release/released/23

# Suppose PREV=57abe0e94 and CURR=a90977260
git log --oneline --decorate --date=short 57abe0e94..a90977260 -- frameworks/
git show --name-only --pretty=format:%H%n%s%n%b 2c94f3ebf -- frameworks/
```

