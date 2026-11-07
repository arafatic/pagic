# Pagic vendored winit

Source repository: https://github.com/arafatic/winit

Exact pinned revision: `a517b3846ee8457d84ee4975110bfda6797c44dc`

Upstream project: https://github.com/rust-windowing/winit

Pagic uses this complete Git tree locally through its root Cargo path patch so
builds and packages do not fetch the former fork repository. The package remains
`winit` version `0.30.13`; its source, features, APIs and behavior are unchanged.
The root package and the original `dpi` workspace member are both retained.
`dpi` used by Pagic continues to resolve from crates.io as before.

All 261 original files, including `LICENSE`, copyright, `PATCHES.md`, manifests,
tests, examples and upstream metadata, are copied byte for byte.
`UPSTREAM-SHA256.json` records every original file. This note and the checksum
inventory are the only additions. Do not format or modify the pinned source.

`vendor/winit` is a separate older provenance copy based on
`e9809ef54b18499bb4f2cac945719ecc2a61061b`. It is retained for existing historical
source guards, is not the runtime dependency, and must not replace this tree.
