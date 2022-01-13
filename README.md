# mocked-segments-generator

## How to release

The releases are automated via GitHub actions, using [this configuration file](https://github.com/Qvant-lab/mocked-segments-generator-rust/blob/main/.github/workflows/delivery.yml).

The release is triggered by pushing a tag.

1. Bump the version in `Cargo.toml` and run the app to update the `lock` file
1. Commit the changes and push them.
1. Create a new tag e.g. `git tag -a v0.7.0`
1. Push the tag `git push --tags`
1. Wait for the build to finish on the [Actions page](https://github.com/Qvant-lab/mocked-segments-generator-rust/actions)
