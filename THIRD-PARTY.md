# Third-party material

Bruma's license ([LICENSE-MIT](LICENSE-MIT) or [LICENSE-APACHE](LICENSE-APACHE), at your option) covers only the work of its contributors. **Third-party material is not part of that licensing**: it keeps the license its authors gave it, and whoever reuses it follows that license.

When you add some (code, fonts, images, logos, map data), list it here with its source and license, keep its license file next to it, and annotate its path in [REUSE.toml](REUSE.toml), the machine-readable version of this list (`reuse lint` checks it in CI; the license texts are in `LICENSES/`).

## Dependencies

Crates (`Cargo.toml`, `Cargo.lock`) are distributed under their own licenses, which each crate carries. `cargo deny check licenses` fails when a dependency has a license that [deny.toml](deny.toml) does not allow; the allowlist is Bruma's own, not Micelio's (ADR-0001, point 8, of `micelio-engineering`).
