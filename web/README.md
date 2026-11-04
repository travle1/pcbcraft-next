# Browser build

The browser target uses the same schematic editor, Electronics session, commands,
undo history and `.pc` codecs as the native application. It runs entirely in the
browser; hosting serves static files. The desktop executable remains independent.

## Run locally

Use Rust 1.95+, Python 3 and the pinned browser build tools:

```sh
rustup target add wasm32-unknown-unknown
cargo install trunk --version 0.21.14 --locked
cargo install wasm-bindgen-cli --version 0.2.130 --locked
cargo fetch --locked
trunk serve
```

Open <http://127.0.0.1:8080/> for an empty project, or add `?demo=1` for the
original fan-controller example. `Trunk.toml` selects `web` without the native
default features. The build hook generates dependency and asset notices from the
locked browser graph; dependencies must be fetched before that offline hook runs.
If your shell exports `NO_COLOR=1`, Trunk 0.21.14 requires `env -u NO_COLOR trunk serve`.

For a static release bundle:

```sh
trunk build --release --locked --offline
python3 -m http.server 8080 --bind 127.0.0.1 --directory target/web-dist
```

The complete distributable is `target/web-dist/`, including `licenses/`. Serve it
over HTTP on localhost or HTTPS on a public host; opening `index.html` as a local
file does not provide the required module loading/browser environment. Asset URLs
are relative, so the same bundle can run under `/pcbcraft-next/` or at a site's root.

## Files and Assistant

**Open draft** reads a local file, rejects files larger than 4 MiB before reading
their contents, then applies the shared format/structure checks. Cancelling or
rejecting an import retains the current project. **Download draft** (Ctrl+S)
encodes the complete project, including all sheets, into a `.pc` download.
The application does not upload project data to a server.

A download request cannot confirm that a file reached disk. It therefore keeps
the session's dirty state and tab-leaving warning. When replacing a changed
project, download and verify the file before choosing **Discard changes** to
continue. Browser warnings depend on browser policy and are not crash recovery;
there is no autosave, persistent browser storage or file overwrite tracking in
this first target. Browser file imports and native saves use the same format.

The Assistant can copy a request containing the actual sheet document and
selection. It does not launch ACP executables or expose the native local MCP
bridge. There is no in-browser model connection or proposal-return transport yet.
Native connection settings and operating-system window controls are omitted.
The built-in example and symbols retain their existing electrical limitations;
ERC, PCB layout and manufacturing remain unimplemented.

## GitHub Pages

[Publish browser app](../.github/workflows/pages.yml) runs manually or when an
explicit `web-preview-*` tag is pushed.
It builds the pinned WASM target and uploads the complete static bundle, then
deploys it using the `github-pages` environment. Ordinary branch pushes do not
deploy. Preview tags allow a feature-branch snapshot to be published before the
workflow reaches the default branch. Each published tag replaces the site at the
same URL; it does not create a separate site per tag.

For initial setup, select **Settings →
Pages → Source → GitHub Actions**. Push an authorized preview tag to publish
that revision. Once the workflow is on the default branch, **Actions → Publish
browser app → Run workflow** is also available. Environment protection may require
approval. GitHub's [custom workflow documentation](https://docs.github.com/en/pages/getting-started-with-github-pages/using-custom-workflows-with-github-pages)
describes those repository settings. No repository settings or live deployment
are changed by a local build.

## Verification

```sh
cargo check --offline --locked --no-default-features --features web --target wasm32-unknown-unknown --bin pcbcraft
cargo clippy --offline --locked --no-default-features --features web --target wasm32-unknown-unknown --bin pcbcraft -- -D warnings
npm install --prefix work/web-test playwright@1.63.0
work/web-test/node_modules/.bin/playwright install chromium
node scripts/test-web.cjs
```

The browser check serves the actual bundle under `/pcbcraft-next/`, exercises the
visible controls, and checks exported project data. It writes screenshots and a
round-trip file to ignored `work/proof/`. Set `PCBCRAFT_CHROMIUM_PATH` to use an
existing Chromium executable. Current qualification is Chromium on Linux at
1600×1000 and 1000×700 with WebGL; other browsers, mobile/touch workflows and
future 3D work are not qualified by this slice.
