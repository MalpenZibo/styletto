//! The system's file chooser, through the desktop portal.

use std::collections::HashMap;
use std::path::PathBuf;

use futures::StreamExt;
use zbus::Connection;
use zbus::{OwnedObjectPath, OwnedVariant, Variant};

#[zbus::proxy(
    interface = "org.freedesktop.portal.FileChooser",
    default_service = "org.freedesktop.portal.Desktop",
    default_path = "/org/freedesktop/portal/desktop"
)]
trait FileChooser {
    fn open_file(
        &self,
        parent_window: &str,
        title: &str,
        options: HashMap<&str, Variant<'_>>,
    ) -> zbus::Result<OwnedObjectPath>;
}

#[zbus::proxy(
    interface = "org.freedesktop.portal.Request",
    default_service = "org.freedesktop.portal.Desktop"
)]
trait Request {
    #[zbus(signal)]
    fn response(&self, response: u32, results: HashMap<String, OwnedVariant>) -> zbus::Result<()>;
}

/// An image file the user picks, or `None` if they cancel.
pub async fn image(title: &str) -> zbus::Result<Option<PathBuf>> {
    let connection = Connection::session().await?;
    // The request's path is known before the call, so its answer can't be
    // missed by listening too late.
    let token = format!("storybook{}", std::process::id());
    let sender = connection
        .unique_name()
        .map(|name| name.trim_start_matches(':').replace('.', "_"))
        .unwrap_or_default();
    let request = RequestProxy::builder(&connection)
        .path(format!(
            "/org/freedesktop/portal/desktop/request/{sender}/{token}"
        ))
        .build()
        .await?;
    let mut responses = request.receive_response().await?;

    let types: Vec<(u32, &str)> = ["image/png", "image/jpeg", "image/webp"]
        .into_iter()
        .map(|mime| (1, mime))
        .collect();
    let options = HashMap::from([
        ("handle_token", Variant::from(token.as_str())),
        ("filters", Variant::from(vec![("Images", types)])),
    ]);
    FileChooserProxy::new(&connection)
        .await?
        .open_file("", title, options)
        .await?;

    let Some(response) = responses.next().await else {
        return Ok(None);
    };
    let args = response.args()?;
    if args.response != 0 {
        return Ok(None);
    }
    let uris = args
        .results
        .get("uris")
        .and_then(|uris| Vec::<String>::try_from(uris.clone()).ok())
        .unwrap_or_default();
    Ok(uris
        .first()
        .and_then(|uri| url::Url::parse(uri).ok())
        .and_then(|url| url.to_file_path().ok()))
}
