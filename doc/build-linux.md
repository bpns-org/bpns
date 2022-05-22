# BUILD FOR LINUX

Before build, see [build requirements](#linux-distribution-specific-instructions) for your specific platform!

Optionally, see [features flag](#features-flag) to enable some cool features.

## To Build

```
cargo build --release
```

## Features flag

To enable features use ```--feature <feature_name>``` (you can append multiple features name).

To disable default features use ```--no-default-features``` argument. 

The following feature flags are available:

| Feature             | Default | Description                                                           |
| ------------------- | :-----: | --------------------------------------------------------------------- |
| `matrix`            |   No    | Enable [Matrix](https://matrix.org) Bot                              |
| `server`            |   Yes   | Server API                                                            |

### Examples

Keep default features and enable Matrix:

```
cargo build --release --feature matrix
```

Disable default features and enable Matrix:

```
cargo build --release --no-default-features --feature matrix
```

## Linux Distribution Specific Instructions

### Ubuntu & Debian

#### Dependency Build Instructions

Build requirements:

```
sudo apt install build-essential cargo clang cmake libssl-dev pkg-config rustc
```

