# minitar
A minimal implementation of the tape archive (tar) format in rust.

## Usage as a library

```rust
minitar = "0.1.0"
```

## Structs

The `tar_header` struct contains a valid tar file header:
```rust
pub struct tar_header {
    file_name: [u8; 100],
    file_mode: [u8; 8],
    own_user: [u8; 8],
    own_group: [u8; 8],
    file_size: [u8; 12],
    mod_time: [u8; 12],
    header_checksum: [u8; 8],
    link_indicator: [u8; 1],
    link_name: [u8; 100],
    ustar_magic: [u8; 6],
    ustar_version: [u8; 2],
    own_user_name: [u8; 32],
    own_group_name: [u8; 32],
    device_major: [u8; 8],
    device_minor: [u8; 8],
    file_prefix: [u8; 155],
    reserved: [u8; 12],
}
```

The `tar_node` struct contains a representation of a file in the tar format:
```rust
pub struct tar_node {
    header: tar_header,
    data: Vec<[u8; 512]>,
}
```

A tar file is comprised of 1 or more `tar_nodes`, represented as a vector, i.e. `Vec<tar_node>`.

## API

Reads in a file and build an internal `Vec<tar_node>` structure:
```rust
pub fn file_read(filename: String) -> Vec<tar_node>
```

Reads in a tar file and build an internal `Vec<tar_node>` structure:
```rust
pub fn tar_read(filename: String) -> Vec<tar_node>
```

Write out an internal `Vec<tar_node>` structure:
```rust
pub fn tar_write(filename: String, tar: &mut Vec<tar_node>)


```
