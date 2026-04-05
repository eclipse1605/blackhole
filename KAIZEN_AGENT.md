# Agent proposal

**Issue:** reduce noise around the disk

there is lob noise around the accretion disk of the blackhole, the goal is to reduce it.

## CLI context hints

### Prioritized files
- legacy/shaders/blackhole_schwarzchild.frag (score 0.32)
- shaders/blackhole.frag (score 0.32)
- shaders/blackhole.vert (score 0.32)

### Discovery commands
```bash
rg --files
rg -n --ignore-case 'reduce|noise|around|disk|there|lob|accretion|blackhole' .
find . -maxdepth 4 -type f | head -n 200
```

### Inspection commands
```bash
sed -n '1,200p' <path-to-file>
```

### Verification commands
```bash
rg -n --ignore-case 'reduce|noise|around|disk|there|lob|accretion|blackhole' .
if [ -f package.json ]; then npm test -- --help || true; fi
if [ -f bun.lockb ] || [ -f bun.lock ]; then bun test --help || true; fi
if [ -f pyproject.toml ] || [ -f pytest.ini ]; then pytest -q || true; fi
```

## Verification checklist
- Keep modifications scoped to prioritized files unless new dependencies are necessary.
- Capture any assumptions or unresolved risks in the PR description.

### Suggested test commands
```bash
rg -n --ignore-case 'reduce|noise|around|disk|there|lob|accretion|blackhole' .
if [ -f package.json ]; then npm test -- --help || true; fi
if [ -f bun.lockb ] || [ -f bun.lock ]; then bun test --help || true; fi
if [ -f pyproject.toml ] || [ -f pytest.ini ]; then pytest -q || true; fi
```

_Updated 2026-04-05T01:22:20.840Z_
