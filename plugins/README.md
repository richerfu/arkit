# Plugins

Concrete, optional integrations live in this directory. Framework and
component crates expose generic extension points; plugins implement those
public contracts and may depend on the owning crates.

Dependency direction must remain one-way:

```text
plugins/* -> crates/*
```

Core crates must not depend on a concrete plugin. Plugin dependencies are
declared in the root workspace and consumed explicitly by applications that
need them.
