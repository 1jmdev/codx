use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use lsp_types::Uri;
use url::Url;

pub(crate) fn path_to_uri(path: &Path) -> Option<Uri> {
    let url = Url::from_file_path(path).ok()?;
    Uri::from_str(url.as_str()).ok()
}

pub(crate) fn uri_to_path(uri: &Uri) -> Option<PathBuf> {
    let url = Url::parse(uri.as_str()).ok()?;
    url.to_file_path().ok()
}
