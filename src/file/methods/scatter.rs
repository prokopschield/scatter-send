use std::{os::unix::ffi::OsStrExt, path::Path};

use anyhow::{anyhow, Result};
use ps_mmap::ReadableMemoryMap;
use ps_str::Utf8Encoder;
use scatter_net::ScatterNet;

use crate::File;

impl File {
    /// Scatters the file located in `path`.
    /// # Errors
    /// - TODO. [`anyhow::Result`] is returned for now.
    pub async fn scatter<P: AsRef<Path> + Send>(path: P, net: ScatterNet) -> Result<Self> {
        let path = path.as_ref();

        let name = path
            .file_name()
            .ok_or_else(|| anyhow!("Invalid filename"))?
            .as_bytes()
            .to_utf8_string();

        let mapping = ReadableMemoryMap::map_path(path)?;
        let size = mapping.len().try_into()?;

        let hkey = net.get_lake().put_blob(&mapping)?.to_string();

        /* TODO
        let hkey = net
            .put_blob(bytes::Bytes::from_owner(mapping))?
            .early_return() // don't wait for network propagation
            .await?;

        let hkey = hkey.to_string();
        */

        Ok(Self { hkey, name, size })
    }
}
