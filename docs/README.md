# Developer Section

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
