# Updating External Examples to Use Latest Bevygap

## Overview

External example repositories that use bevygap need to be updated to reference the latest version from `bananabit-dev/bevygap` instead of the old `RJ/bevygap` repository.

## bevygap-spaceships Repository Updates

The [bevygap-spaceships](https://github.com/RJ/bevygap-spaceships) repository needs the following updates in its `Cargo.toml`:

### Current (Outdated) Dependencies:
```toml
bevygap_client_plugin = {git = "https://github.com/RJ/bevygap.git", tag = "v0.1.7"}
bevygap_server_plugin = {git = "https://github.com/RJ/bevygap.git", tag = "v0.1.7"}
```

### Updated Dependencies:
```toml
bevygap_client_plugin = {git = "https://github.com/bananabit-dev/bevygap.git", rev = "1921c864b3414f45e679894733c2e19fe5e0ecfd"}
bevygap_server_plugin = {git = "https://github.com/bananabit-dev/bevygap.git", rev = "1921c864b3414f45e679894733c2e19fe5e0ecfd"}
```

**Note:** The rev `1921c864b3414f45e679894733c2e19fe5e0ecfd` is the latest commit on the main branch of bananabit-dev/bevygap at the time of this update. For the absolute latest changes including recent example fixes, you may also use rev `4127d21b86ee7d1f8dd7aa1d4973e53b4fbb3337`.

### Alternative: Use Main Branch (Latest)
If you want to always use the latest version:
```toml
bevygap_client_plugin = {git = "https://github.com/bananabit-dev/bevygap.git", branch = "main"}
bevygap_server_plugin = {git = "https://github.com/bananabit-dev/bevygap.git", branch = "main"}
```

## Required Actions

1. **For repository owners**: Update the dependencies in external example repositories to use `bananabit-dev/bevygap`
2. **For users following the documentation**: Use the new repository URL when setting up examples

## API Changes to Be Aware Of

When updating to the latest bevygap, you may need to make the following code changes:

### bevygap_client_plugin Examples
If you have code that uses lightyear's `ClientConfig`:
```rust
// OLD - No longer works
use lightyear::prelude::client::ClientConfig;
app.insert_resource(ClientConfig::default());

// NEW - Use BevygapClientConfig instead
use bevygap_client_plugin::prelude::*;
app.insert_resource(BevygapClientConfig::default());
```

### bevy_nfws Examples
If you have examples that use logging:
```rust
// OLD - Missing import
use bevy::prelude::*;
use bevy_nfws::prelude::*;

// NEW - Add log import
use bevy::prelude::*;
use bevy_nfws::prelude::*;
use log::info;
```

## Version Information

- Repository: `bananabit-dev/bevygap`
- Latest commit: `1921c864b3414f45e679894733c2e19fe5e0ecfd`
- Current version: `0.3.1`

## Testing

After updating dependencies, make sure to:
1. Run `cargo check` to verify compilation
2. Run `cargo build` to ensure all dependencies resolve correctly
3. Test the examples to ensure they work with the updated dependencies