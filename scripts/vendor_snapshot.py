#!/usr/bin/env python3
"""Create a self-contained cargo home snapshot for offline builds.

Copies everything `cargo build --offline --locked` needs for THIS project
from the local cargo home (~/.cargo) into <root>/vendor-snapshot/.cargo:

  - registry/{index,cache,src}   crates pinned in Cargo.lock
  - git/{db,checkouts}           git dependencies (including private repos)

Builds then run fully offline and without network access to the git remotes:

  CARGO_HOME=vendor-snapshot/.cargo cargo build --offline --locked

Cargo resolves the full target-independent graph, so every crate pinned in
Cargo.lock (crates.io src + cache, plus the git checkouts) is copied. This
also works around `cargo vendor`, which cannot handle Cargo.lock files where
the same package is reachable through two source spellings of the same git
URL (e.g. pop-os/libcosmic), and keeps private git dependencies buildable.
"""

import glob
import os
import re
import shutil
import sys
import tomllib

REGISTRY_SRC = re.compile(r'/registry/src/(index[^/]+/[A-Za-z0-9_.!+-]+(?:-[A-Za-z0-9_.!+-]+)?)/')
GIT_CHECKOUT = re.compile(r'/git/checkouts/([A-Za-z0-9_.!+-]+/[A-Za-z0-9_.!+-]+)/')


def copytree(src, dst):
    if os.path.exists(src) and not os.path.exists(dst):
        if os.path.isdir(src):
            shutil.copytree(src, dst, symlinks=True)
        else:
            os.makedirs(os.path.dirname(dst), exist_ok=True)
            shutil.copy2(src, dst)


def crates_from_build_artifacts(target, home):
    """Crate source dirs actually compiled for this host, from *.d files."""
    src_crates = set()
    git_crates = set()
    home = home.rstrip('/')
    for dfile in glob.glob(os.path.join(target, '*', 'deps', '*.d')):
        try:
            with open(dfile, encoding='utf-8', errors='replace') as f:
                content = f.read()
        except OSError:
            continue
        src_crates.update(f'{home}/registry/src/{m}' for m in REGISTRY_SRC.findall(content))
        git_crates.update(f'{home}/git/checkouts/{m}' for m in GIT_CHECKOUT.findall(content))
    return src_crates, git_crates


def main(root='.'):
    root = os.path.abspath(root)
    lock = tomllib.load(open(os.path.join(root, 'Cargo.lock'), 'rb'))
    home = os.path.expanduser(os.environ.get('CARGO_HOME', '~/.cargo'))
    out = os.path.join(root, 'vendor-snapshot', '.cargo')
    snap = os.path.dirname(out)
    if os.path.exists(snap):
        shutil.rmtree(snap)

    home_rel = home.rstrip('/')

    # Registry index: needed for offline resolution; it is small, keep it all.
    for idx in glob.glob(os.path.join(home, 'registry', 'index', '*')):
        copytree(idx, os.path.join(out, 'registry', 'index', os.path.basename(idx)))

    # Which crate source dirs to copy: prefer the compiled set (accurate, small),
    # fall back to everything in Cargo.lock (complete, large).
    compiled_src, compiled_git = crates_from_build_artifacts(os.path.join(root, 'target'), home)
    locked_src = {
        f'{home_rel}/registry/src/{idx}/{p["name"]}-{p["version"]}'
        for p in lock['package']
        if p.get('source', '').startswith('registry+')
        for idx in os.listdir(os.path.join(home, 'registry', 'src'))
        if os.path.isdir(os.path.join(home, 'registry', 'src', idx, f'{p["name"]}-{p["version"]}'))
    }
    locked_git = {
        f'{home_rel}/git/checkouts/{d}/{c}'
        for d in os.listdir(os.path.join(home, 'git', 'checkouts'))
        if os.path.isdir(os.path.join(home, 'git', 'checkouts', d))
        for c in os.listdir(os.path.join(home, 'git', 'checkouts', d))
    }
    # cargo resolves the full (target-independent) graph and needs the sources
    # of every locked crate, so use everything in Cargo.lock; the compiled set
    # is added as a safety net in case it is missing from the lock.
    want_src = locked_src | compiled_src
    want_git = locked_git | compiled_git

    for cand in sorted(want_src):
        name = cand.rsplit('/', 1)[1]
        idx = cand.rsplit('/', 2)[1]
        copytree(cand, os.path.join(out, 'registry', 'src', idx, name))
        crate_file = os.path.join(home, 'registry', 'cache', idx, f'{name}.crate')
        if os.path.exists(crate_file):
            copytree(crate_file, os.path.join(out, 'registry', 'cache', idx, f'{name}.crate'))

    # Git deps: keep the bare caches (small, all of them) plus referenced checkouts.
    db = os.path.join(home, 'git', 'db')
    if os.path.isdir(db):
        copytree(db, os.path.join(out, 'git', 'db'))
    for cand in sorted(want_git):
        rel = cand[len(home_rel) + 1:]  # git/checkouts/<repo>/<commit>
        copytree(cand, os.path.join(out, rel))

    total = 0
    for dirpath, _, files in os.walk(snap):
        for f in files:
            total += os.path.getsize(os.path.join(dirpath, f))
    print(f'vendor-snapshot: {total / 1e6:.0f} MB in {snap}')


if __name__ == '__main__':
    main(sys.argv[1] if len(sys.argv) > 1 else '.')
