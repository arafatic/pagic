# Verified license sources

Recto's initial commit `42938304f695a1883669117ec5aee485588ca2a5` and current upstream Cargo metadata declare
`MIT OR Apache-2.0`. LICENSE and LICENSE-MIT contain the original MIT text; those originals
remain unchanged. The missing Apache counterpart is supplied verbatim from the Apache
Software Foundation, without changing the existing Cargo declaration or adding custom terms.
README, Credits & Licenses and native credits now describe that existing declaration consistently.

- LICENSE-APACHE: https://www.apache.org/licenses/LICENSE-2.0.txt
  SHA-256: `cfc7749b96f63bd31c3c42b5c471bf756814053e847c10f3eb003417bc523d30`.
- licenses/egui-MIT.txt: https://github.com/arafatic/egui/blob/6d518fc799681571963842f95b55a6b68431b15a/LICENSE-MIT
  Git blob: `673ea5f0505d6fa65c6a2b6a753c93b8e19d11b9`; SHA-256: `95ca92f5f8ea5231f1580b3a2a799e8260af3114b900e1def5355a7f44bcf60c`.
  Verified against the exact dependency checkout; its authorship notice is unchanged.
- Original metadata: https://github.com/arafatic/recto/blob/42938304f695a1883669117ec5aee485588ca2a5/Cargo.toml
- Original MIT license: https://github.com/arafatic/recto/blob/42938304f695a1883669117ec5aee485588ca2a5/LICENSE

Existing checked-in hayro, Lucide, font OFL and historical Recto notices are embedded verbatim
by Credits & Licenses. These do not license the newly supplied Pagic artwork: its separate
provenance and unresolved distribution terms remain unchanged. This correction does not claim
to complete the broader transitive-dependency/NOTICE packaging inventory.

## Verified upstream dependency notices (`licenses/upstream/`)

Some crates.io packages published from multi-crate Cargo workspaces omit the repository-root
license files from their subdirectory tarballs. For the Windows MSIX dependency inventory
(`packaging/windows/msix/msix.py`), `licenses/upstream-notices.json` and `licenses/upstream/`
supply the verbatim license files from each crate's exact `.cargo_vcs_info.json` Git commit,
verified by Git blob SHA-1 and file SHA-256:

- `AccessKit/accesskit` (`accesskit@0.25.2`, `accesskit_consumer@0.39.2`, `accesskit_windows@0.35.2`, `accesskit_winit@0.34.2`) at `f3026222559c7f5f46d8f25cb7404de68eb149f0`:
  - `LICENSE-APACHE`: https://github.com/AccessKit/accesskit/blob/f3026222559c7f5f46d8f25cb7404de68eb149f0/LICENSE-APACHE
    Git blob: `1b5ec8b78e237b5c3b3d812a7c0a6589d0f7161d`; SHA-256: `62c7a1e35f56406896d7aa7ca52d0cc0d272ac022b5d2796e7d6905db8a3636a`.
  - `LICENSE-MIT`: https://github.com/AccessKit/accesskit/blob/f3026222559c7f5f46d8f25cb7404de68eb149f0/LICENSE-MIT
    Git blob: `31aa79387f27e730e33d871925e152e35e428031`; SHA-256: `23f18e03dc49df91622fe2a76176497404e46ced8a715d9d2b67a7446571cca3`.
  - `LICENSE.chromium`: https://github.com/AccessKit/accesskit/blob/f3026222559c7f5f46d8f25cb7404de68eb149f0/LICENSE.chromium
    Git blob: `a32e00ce6be3620adca152e53734f7de8fda38cf`; SHA-256: `845022e0c1db1abb41a6ba4cd3c4b674ec290f3359d9d3c78ae558d4c0ed9308`.
- `dropbox/rust-alloc-no-stdlib` (`alloc-stdlib@0.2.4`) at `ae42d22078b98549e987d2f03d12df7b984fde47`:
  - `LICENSE`: https://github.com/dropbox/rust-alloc-no-stdlib/blob/ae42d22078b98549e987d2f03d12df7b984fde47/LICENSE
    Git blob: `cd496ba72eb37e83a44958358d1f89a8a28cbc15`; SHA-256: `c0c56f26d9c051cac4d200c34c84e7ae9aaa853e01a982a1df08b09931e518ae`.
- `boa-dev/boa` (`boa_ast@0.22.0`, `boa_engine@0.22.0`, `boa_gc@0.22.0`, `boa_interner@0.22.0`, `boa_macros@0.22.0`, `boa_parser@0.22.0`, `boa_string@0.22.0` at `337a3668a0dc86dd401ea20906e782249a64a228`; `small_btree@0.1.0`, `tag_ptr@0.1.0` at `ad8739f5e0b51d20faf7a2cce98afa5c40121438`):
  - `LICENSE-MIT`: https://github.com/boa-dev/boa/blob/337a3668a0dc86dd401ea20906e782249a64a228/LICENSE-MIT
    Git blob: `d87a9e81ddc8b3e77715796fa28b983c3f9ee4b1`; SHA-256: `60d7cf4b66cef6bab24701e25f87829c4ce01fb734def53c41e5ed79ab434cb3`.
  - `LICENSE-UNLICENSE`: https://github.com/boa-dev/boa/blob/337a3668a0dc86dd401ea20906e782249a64a228/LICENSE-UNLICENSE
    Git blob: `cf1ab25da0349f84a3fdd40032f0ce99db813b8b`; SHA-256: `88d9b4eb60579c191ec391ca04c16130572d7eedc4a86daa58bf28c6e14c9bcd`.
- `DoumanAsh/clipboard-win` (`clipboard-win@5.4.3`) at `ed27008ae5d1f10cfbb266de039f0fb35fd05a56`:
  - `LICENSE`: https://github.com/DoumanAsh/clipboard-win/blob/ed27008ae5d1f10cfbb266de039f0fb35fd05a56/LICENSE
    Git blob: `36b7cd93cdfbac762f5be4c6ce276df2ea6305c2`; SHA-256: `c9bff75738922193e67fa726fa225535870d2aa1059f91452c411736284ad566`.
- `RustCrypto/formats` (`cms@0.2.3`) at `5821a21553509dbd03eae593b0a1fad4e2083d4e`:
  - `der/LICENSE-APACHE`: https://github.com/RustCrypto/formats/blob/5821a21553509dbd03eae593b0a1fad4e2083d4e/der/LICENSE-APACHE
    Git blob: `78173fa2e753d614bb0aa5ad125ddbbe70b282c4`; SHA-256: `a9040321c3712d8fd0b09cf52b17445de04a23a10165049ae187cd39e5c86be5`.
  - `der/LICENSE-MIT`: https://github.com/RustCrypto/formats/blob/5821a21553509dbd03eae593b0a1fad4e2083d4e/der/LICENSE-MIT
    Git blob: `e0d082780149ee3ce9f69b764dd138f757f2b62c`; SHA-256: `ad64fcb9589f162720f3cc5010ad76ca6ad3764e11861f9192c489df176bb71d`.
- `google/flatbuffers` (`flatbuffers@24.12.23`) at `a2cd1ea3b6d3fee220106b5fed3f7ce8da9eb757`:
  - `LICENSE`: https://github.com/google/flatbuffers/blob/a2cd1ea3b6d3fee220106b5fed3f7ce8da9eb757/LICENSE
    Git blob: `d645695673349e3947e8e5ae42332d0ac3164cd7`; SHA-256: `cfc7749b96f63bd31c3c42b5c471bf756814053e847c10f3eb003417bc523d30`.
- `brendanzab/gl-rs` (`gl_generator@0.14.0` at `ea503e8d5fb6d73c6030e6191ce738cd3bf3433e`; `khronos_api@3.1.0` at `f150967b1c44ae888e6676f93f639ebc82771bdc`):
  - `LICENSE`: https://github.com/brendanzab/gl-rs/blob/ea503e8d5fb6d73c6030e6191ce738cd3bf3433e/LICENSE
    Git blob: `d645695673349e3947e8e5ae42332d0ac3164cd7`; SHA-256: `cfc7749b96f63bd31c3c42b5c471bf756814053e847c10f3eb003417bc523d30`.
- `robertknight/ocrs` (`ocrs@0.13.1`) at `e5f1c5205326804637368422fe420aa1b9769676`:
  - `LICENSE-APACHE.txt`: https://github.com/robertknight/ocrs/blob/e5f1c5205326804637368422fe420aa1b9769676/LICENSE-APACHE.txt
    Git blob: `1b5ec8b78e237b5c3b3d812a7c0a6589d0f7161d`; SHA-256: `62c7a1e35f56406896d7aa7ca52d0cc0d272ac022b5d2796e7d6905db8a3636a`.
  - `LICENSE-MIT.txt`: https://github.com/robertknight/ocrs/blob/e5f1c5205326804637368422fe420aa1b9769676/LICENSE-MIT.txt
    Git blob: `4fe232785fbf61fe9000d206d248bbefda37c307`; SHA-256: `96b5371602fbd3cb11a680dade714465520c72b02397be5095a3c29b1a3fb30a`.
- `aclysma/profiling` (`profiling@1.0.18`) at `8271551172eb6fa4cba47369aedd93790c623df9`:
  - `LICENSE-APACHE`: https://github.com/aclysma/profiling/blob/8271551172eb6fa4cba47369aedd93790c623df9/LICENSE-APACHE
    Git blob: `139dce95d473c201a8561ef17536314d8b8cda65`; SHA-256: `10d30a673cd5e9349bdc02aeb48f14b3386d27d0da32df8f0a555d4aa16aa551`.
  - `LICENSE-MIT`: https://github.com/aclysma/profiling/blob/8271551172eb6fa4cba47369aedd93790c623df9/LICENSE-MIT
    Git blob: `6a84e059ad8b43e03c2b06bc2af4dc1d8442808d`; SHA-256: `c8167fdeeed46d3f244d3f85c5bf998ce889343691c32be2c61a8bc4b5c08333`.
- `robertknight/rten` (`rten@0.26.0`, `rten-base@0.26.0`, `rten-gemm@0.26.0`, `rten-imageproc@0.26.0`, `rten-model-file@0.26.0`, `rten-onnx@0.26.0`, `rten-parallel@0.26.0`, `rten-shape-inference@0.26.0`, `rten-simd@0.26.0`, `rten-tensor@0.26.0`, `rten-vecmath@0.26.0`) at `14cbf4620476f402237bb29f731a244d3b6bed4d`:
  - `LICENSE-APACHE.txt`: https://github.com/robertknight/rten/blob/14cbf4620476f402237bb29f731a244d3b6bed4d/LICENSE-APACHE.txt
    Git blob: `1b5ec8b78e237b5c3b3d812a7c0a6589d0f7161d`; SHA-256: `62c7a1e35f56406896d7aa7ca52d0cc0d272ac022b5d2796e7d6905db8a3636a`.
  - `LICENSE-MIT.txt`: https://github.com/robertknight/rten/blob/14cbf4620476f402237bb29f731a244d3b6bed4d/LICENSE-MIT.txt
    Git blob: `706fc8c2382ae463729fdcdcf559dbf0319e0b0a`; SHA-256: `5096967f408fab8da02f4b6dfac6b00ccc19ca17c635f9009ee74594ac275e18`.
- `prefix-dev/sigstore-rust` (`sigstore-bundle@0.6.6`, `sigstore-crypto@0.6.6`, `sigstore-merkle@0.6.6`, `sigstore-rekor@0.6.6`, `sigstore-trust-root@0.6.6`, `sigstore-tsa@0.6.6`, `sigstore-types@0.6.6`, `sigstore-verify@0.6.6`) at `c9d76063833cb58a06483b181096294524d2dbf1`:
  - `LICENSE`: https://github.com/prefix-dev/sigstore-rust/blob/c9d76063833cb58a06483b181096294524d2dbf1/LICENSE
    Git blob: `261eeb9e9f8b2b4b0d119366dda99c6fd7d35c64`; SHA-256: `c71d239df91726fc519c6eb72d318ec65820627232b2f796219e87dcf35d0ab4`.
- `gfx-rs/rspirv` (`spirv@0.4.0+sdk-1.4.341.0`) at `8afc3d0ac8e158128cd1410bb2e4b4c26ab11bb4`:
  - `LICENSE`: https://github.com/gfx-rs/rspirv/blob/8afc3d0ac8e158128cd1410bb2e4b4c26ab11bb4/LICENSE
    Git blob: `d645695673349e3947e8e5ae42332d0ac3164cd7`; SHA-256: `cfc7749b96f63bd31c3c42b5c471bf756814053e847c10f3eb003417bc523d30`.
- `enum-map@3.1.0` (`a09d924a4afde4e2864a1df7b1438a5e6cdf3e7e`) and `enum-map-derive@1.0.1` (`ccf562d340723530d54f7bdfeb1782b4ded803a7`) ship REUSE-compliant `LICENSES/{Apache-2.0.txt,CC0-1.0.txt,MIT.txt}` and `SPDX-FileCopyrightText` headers in `README.md` directly inside their crates.io tarballs; `msix.py` stages those in-crate files verbatim.
- Three crates (`fmt-derive@0.1.2` and `fmt-derive-proc@0.1.2` from `danielschemmel/fmt-derive@3f1abe44dc5e36e467565d3bf636faba5193f2a1`, and `rustls-cng@0.7.1` from `rustls/rustls-cng@b9126716029a105a08c1786993922103d9865bad`) have no standalone `LICENSE` file in their upstream Git repositories at those pinned commits. Their license declarations and authorship are formally verified and accepted directly from their upstream `Cargo.toml.orig` and `README.md` metadata without inventing standalone `LICENSE` files:
  - `fmt-derive@0.1.2`: `authors = ["Daniel Schemmel <daniel@schemmel.net>"]`, `license = "MIT OR Apache-2.0"`. `Cargo.toml.orig` SHA-256: `9c189a45851c073e2a44c2a3ca7899fbd8e1afa20586111f6da4e0878e4e9409`, `README.md` SHA-256: `c0f60a951b9081918334fb15439a23565437103ccb7f9e788df246b9d0f4e63c`.
  - `fmt-derive-proc@0.1.2`: `authors = ["Daniel Schemmel <daniel@schemmel.net>"]`, `license = "MIT OR Apache-2.0"`. `Cargo.toml.orig` SHA-256: `5712e5a55f0fa6650aa603686d68d22989700ec9d4963e0b1a7f422bd594791b`, `README.md` SHA-256: `47abfd71107ac0de38708c3e6c06b28129b55c62f49b1147cc4631e06892b310`.
  - `rustls-cng@0.7.1`: `authors = ["Dmitry Pankratov <dmitry@pankratov.net>"]`, `license = "MIT/Apache-2.0"`. `Cargo.toml.orig` SHA-256: `b0ebe8f044e61d675100e28feb2fd47275e469cd646627c33db6857390c93d66`, `README.md` SHA-256: `3f30ebca33c1756ac89caafa5224ecca5e5de61175680372f7c8bb4955bb996d`.
  Their verbatim upstream metadata files are staged into the package and hash-verified by `msix.py`, satisfying notice obligations under both MIT and Apache-2.0 terms.
