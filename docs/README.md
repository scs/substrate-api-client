# Developer Section

## TODO List for a new Release
If a new release is released, the following tasks need to be done:
1. Update the .toml versions in accordance with the [Semantic Versioning](../README.md#version-numbers) section.
2. Add a new tag to the desired commit, see the [Tag generation](#tag-generation) section.
3. Update and release the new release.
4. Create a new branch and change all github deps to crates.io deps. See https://github.com/scs/substrate-api-client/issues/528 for an example update. The [psvm](https://crates.io/crates/psvm) tool can be useful for updating the polkadot dependencies.
5. Publish to crates.io, see https://doc.rust-lang.org/cargo/reference/publishing.html for more info.


## Automatic Release generation

A new draft release gets generated upon a new tag with a name matching `v[0-9]+.[0-9]+.[0-9]+*` (E.g. v1.2.3 and v1.2.3-rc1)
See the workflow: https://github.com/scs/substrate-api-client/blob/master/.github/workflows/draft-release.yml

Example release: https://github.com/scs/substrate-api-client/releases/tag/v0.10.0

The `🎉 Featuring` section has been created manually. It should show the user directly what has been updated and what new features have been added (not only PR names)

### PR Labels
For automatic release generation, `E` and `F` labels are used.

They have the following meaning:

`E` labels (enforced by CI):
- `E0-silent`: PR will not be mentioned at all in the release text. This should only be used for PRs that do not change any piece of functional code. E.g. CI and documentation updates.
- `E1-breaksnothing`: PR will be listed in release text, no special release category.
- `E1-breaksapi`: PR will be listed in release text in the category of `Breaking Changes`. Api-client users should pay special attention to this PR, as they most likely need to adapt their existing code.

`F` labels: not enforced by CI, but some labels have a dedicated category in the release:
- `F8-newfeature` and `F7-enhancement` labeled PRs are put in the category `🌈 Features`
- `F2-bug` labeled PRs are put in the category `🐛 Bug Fixes`

All PRs, except for `E0-silent` labaled ones, will be listed in the `Miscellaneous` category.

Check out https://github.com/scs/substrate-api-client/blob/master/.github/release-drafter.yml for more information.


### Tag generation
local tagging (ensure you're on the commit you want the tag to be on)
```
# Create local tag
git tag <TAG NAME> # E.g. v0.10.0
# Push to remote
git push --tags
```
CI should now automatically create a draft release. This draft release needs to be released manually.

## Runtime wasm generation
A new runtime wasm file for CI testing currently needs to be built locally. To do this the following steps need to be done:
1. Download a Polkadot / Substrate node. This can be any up to date node. The following is recommended because it's small and builds fast: https://github.com/paritytech/polkadot-sdk-minimal-template. But it does not include many pallets.

2. Update the runtime names and spec version. The `spec_name` and `impl_name` need to match the original runtime name of the running node. The `spec_version` needs to be higher than the original one.
This needs to be adapted in the source code and looks like the code posted below. Often, it can be found in the `runtime/src/lib.rs` file (Example path minimal runtime: https://github.com/paritytech/polkadot-sdk-minimal-template/blob/master/runtime/src/lib.rs)
```rust
/// The runtime version.
#[runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: create_runtime_str!("<ADAPT THIS NAME>"),
	impl_name: create_runtime_str!("<ADAPT THIS NAME>"),
	authoring_version: 1,
	spec_version: "<INCREMENT THIS VERSION>",
	impl_version: 1,
	apis: RUNTIME_API_VERSIONS,
	transaction_version: 1,
	state_version: 1,
};
```

3. Build the runtime with cargo build. For the minimal runtime it is: `cargo build -p minimal-template-node --release`


4. Get the wasm file from the `target/release/wbuild/<RUNTIME NAME>` folder. Example for the minimal runtime: `~/polkadot-sdk-minimal-template/target/release/wbuild/minimal-template-runtime/minimal_template_runtime.compact.compressed.wasm`

## Cargo.toml dependency Specification
In the `Cargo.toml` we handle the versioning as following:
- By default: No patch version specified
- If there is a specific reason to specify a version with patch, then add a comment why this is needed (security vulnerability, ...)
Cargo will ensure the imported version is not below the specified one (above is possible though).


## Code overview
<p align="center">
<img src=./overview_code_structure.svg width = 700>
</p>
