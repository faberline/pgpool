---
id: apps-pgpool-build-rs
summary: Lossless rust-source-unit coverage for `build.rs`.
fill_sections: [rust-source-unit, changes]
---

# Fillback build.rs

## Source
<!-- type: rust-source-unit lang: rust -->

```rust
// <HANDWRITE gap="missing-generator:project-bootstrap" tracker="#pgpool-bootstrap" reason="Initial build stamp for the working-name app scaffold.">
fn main() {
    build_stamp::stamp("PGPOOL");
}
// </HANDWRITE>
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: "build.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      Lossless rust-source-unit ownership created from explicit file fillback.
```
