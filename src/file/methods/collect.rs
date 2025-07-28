use std::{fs, path::Path};

use anyhow::Result;
use ps_hkey::Hkey;
use ps_mmap::WritableMemoryMap;
use scatter_net::ScatterNet;

use crate::File;

const MB: usize = 1024 * 1024;

impl File {
    /// Collects a file and stores it into the local filesystem at `path`.
    /// # Errors
    /// - TODO. [`anyhow::Result`] is returned for now.
    pub async fn collect<P: AsRef<Path> + Send>(
        &self,
        net: ScatterNet,
        path: P,
    ) -> Result<WritableMemoryMap> {
        let hkey = Hkey::parse(self.hkey.as_bytes());
        let size = usize::try_from(self.size)?;

        let file = fs::OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .truncate(false)
            .open(path)?;

        file.set_len(self.size)?;

        let mapping = WritableMemoryMap::map_file(file)?;

        let segments = (0..size.div_ceil(MB)).map(|start| {
            let net = net.clone();
            let hkey = hkey.clone();
            let mapping = mapping.clone();

            let start = start.saturating_mul(MB);
            let end = start.saturating_add(1).saturating_mul(MB).min(size);
            let range = start..end;

            async move {
                let future = hkey.resolve_slice_async_box(&net, range.clone());
                let segment = future.await?;

                mapping.write()[range].copy_from_slice(&segment);

                anyhow::Ok(())
            }
        });

        for segment in segments {
            segment.await?;
        }

        Ok(mapping)
    }
}
