//! The colour scheme the user prefers, as the desktop portal's
//! `org.freedesktop.appearance` reports it, and every change to it.

use futures::channel::mpsc;
use futures::{SinkExt, Stream, StreamExt, stream};
use log::{error, warn};
use zbus::OwnedVariant;

const NAMESPACE: &str = "org.freedesktop.appearance";
const KEY: &str = "color-scheme";

#[zbus::proxy(
    interface = "org.freedesktop.portal.Settings",
    default_service = "org.freedesktop.portal.Desktop",
    default_path = "/org/freedesktop/portal/desktop"
)]
trait Settings {
    fn read_one(&self, namespace: &str, key: &str) -> zbus::Result<OwnedVariant>;

    #[zbus(signal)]
    fn setting_changed(&self, namespace: &str, key: &str, value: OwnedVariant) -> zbus::Result<()>;
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum ColorScheme {
    /// The user said nothing, or there is no portal to ask.
    #[default]
    NoPreference,
    Dark,
    Light,
}

impl ColorScheme {
    fn from_value(value: &OwnedVariant) -> Self {
        match u32::try_from(value) {
            Ok(1) => ColorScheme::Dark,
            Ok(2) => ColorScheme::Light,
            _ => ColorScheme::NoPreference,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Appearance {
    conn: zbus::Connection,
}

impl Appearance {
    pub async fn connect() -> anyhow::Result<Self> {
        Ok(Self {
            conn: zbus::Connection::session().await?,
        })
    }

    /// The scheme now, then every change.
    pub fn updates(&self) -> impl Stream<Item = ColorScheme> + Send + 'static {
        let conn = self.conn.clone();

        channel(8, |mut output| async move {
            let proxy = match SettingsProxy::new(&conn).await {
                Ok(proxy) => proxy,
                Err(err) => {
                    error!("Failed to reach the settings portal: {err}");
                    return;
                }
            };
            let mut changes = match proxy.receive_setting_changed().await {
                Ok(changes) => changes,
                Err(err) => {
                    error!("Failed to subscribe to SettingChanged: {err}");
                    return;
                }
            };
            let now = match proxy.read_one(NAMESPACE, KEY).await {
                Ok(value) => ColorScheme::from_value(&value),
                Err(err) => {
                    warn!("The portal has no colour scheme: {err}");
                    ColorScheme::NoPreference
                }
            };
            if output.send(now).await.is_err() {
                return;
            }
            while let Some(change) = changes.next().await {
                let Ok(args) = change.args() else { continue };
                if args.namespace == NAMESPACE
                    && args.key == KEY
                    && output
                        .send(ColorScheme::from_value(&args.value))
                        .await
                        .is_err()
                {
                    return;
                }
            }
        })
    }
}

fn channel<T, F>(size: usize, f: impl FnOnce(mpsc::Sender<T>) -> F) -> impl Stream<Item = T>
where
    F: Future<Output = ()>,
{
    let (sender, receiver) = mpsc::channel(size);
    let runner = stream::once(f(sender)).filter_map(|_| async { None });

    stream::select(receiver, runner)
}
