# Developer Section

## TODO List for a new Release
If a new release is released, the following tasks need to be done:
1. Update the .toml versions in accordance with the [Semantic Versioning](#semantic-versioning) section.
2. Add a new tag to the desired commit, see the [Tag generation](#tag-generation) section.
3. Update and release the new release.
4. Publish to crates.io, see https://doc.rust-lang.org/cargo/reference/publishing.html for more info. Important: Create a new branch and change all github deps to crates.io deps. See https://github.com/scs/substrate-api-client/issues/528 for an example update.

## TODO list for a new Parity release branch
To follow a new partiy branch release, the following needs to be done:
1. Create a new branch with the same name as the parity release and update dependencies. See this [commit](https://github.com/scs/substrate-api-client/commit/a50833a922ff98ae59e2fc587e0ab5466b3acab2) for an example update. Release branches are based on a specific api-client release, not master.
2. Push the changes on the new branch and check CI result. CI will be triggered automatically if the branch follows the naming scheme of `release-polkadot-v[0-9]+.[0-9]+.[0-9]+*`. Results can be looked up [here](https://github.com/scs/substrate-api-client/actions)
3. After CI passed, update the associated release text with the new branch. See [this release](https://github.com/scs/substrate-api-client/releases/tag/v0.16.0) as an example.


## Automatic Release generation

A new draft release gets generated upon a new tag with a name matching `v[0-9]+.[0-9]+.[0-9]+*` (E.g. v1.2.3 and v1.2.3-rc1)
See the workflow: https://github.com/scs/substrate-api-client/blob/master/.github/workflows/draft-release.yml

Example release: https://github.com/scs/substrate-api-client/releases/tag/v0.10.0

The `🎉 Featuring` section has been created manually. It should show the user directly what has been updated and what new features have been added (not only PR names)

### Semantic Versioning
This is not yet checked or automated by CI, but it may be in the future. Currently, the api-client is following the semantic versioning în pre-release as the public API may change any time.
In case a new release is triggered, the `.toml` versions of the crates, that have been updated since the last release, should be increased.
- In case of breaking API changes, the minor version should be increased (`0.9.3 -> 0.10.0`)
- In case of changes, which do not break the API, the patch version should be increased (`0.9.3 -> 0.9.4`).
The version of the main .toml should be same as the one of the release.

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

## Code overview
<p align="center">
<img src=./overview_code_structure.svg width = 700>
</p>
