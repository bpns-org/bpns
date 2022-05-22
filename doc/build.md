# BUILD

## Download source code

```
git clone https://github.com/p2kishimoto/bpns && cd bpns
```

## Verify commits

Import gpg keys:

```
gpg --keyserver hkps://keys.openpgp.org --recv-keys $(<contrib/verify-commits/trusted-keys)
```

Verify commit:

```
git verify-commit HEAD
```

## Build

Follow instruction for your OS:

* [Linux](build-linux.md) 
* [OSX](build-osx.md) 