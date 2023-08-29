use std::{fmt::Debug, sync::Arc};

use mavlink::{
    ardupilotmega::MavMessage,
    error::{MessageReadError, MessageWriteError},
    MavConnection, MavHeader,
};
use tokio::task::JoinHandle;

#[derive(Debug)]
pub enum MavlinkConnectionError {
    Timeout,
    ReadError(MessageReadError),
    WriteError(MessageWriteError),
    Other(String),
}

pub enum FilterRes<T> {
    Ready(Option<T>),
    NotReady,
}

#[async_trait::async_trait]
pub trait MavlinkConnection {
    fn send(&self, msg: &MavMessage) -> Result<usize, MavlinkConnectionError>;
    async fn send_wait<R>(
        self: Arc<Self>,
        msg: &MavMessage,
        timeout: std::time::Duration,
        filter: impl Fn(MavMessage) -> FilterRes<R> + Send + Sync + 'static,
    ) -> Result<Option<R>, MavlinkConnectionError>
    where
        R: Send + Sync + 'static;

    // Some == Still Monitoring
    // None == Done Monitoring
    fn monitor(
        self: Arc<Self>,
        timeout: Option<std::time::Duration>,
        monitor: impl Fn(MavMessage) -> Option<()> + Send + Sync + 'static,
    ) -> JoinHandle<Result<(), MavlinkConnectionError>>;

    async fn _receive(self: Arc<Self>) -> Result<(MavHeader, MavMessage), MavlinkConnectionError>;

    fn target_system(&self) -> u8 {
        env!("MAV_TARGET_SYSTEM").parse().unwrap()
    }
    fn target_component(&self) -> u8 {
        env!("MAV_TARGET_COMPONENT").parse().unwrap()
    }

    fn validate(&self, _header: MavHeader) -> bool {
        true
    }
}

#[async_trait::async_trait]
impl<T> MavlinkConnection for Box<T>
where
    T: MavConnection<MavMessage> + Send + Sync + 'static,
{
    #[tracing::instrument(skip(self))]
    fn send(&self, msg: &MavMessage) -> Result<usize, MavlinkConnectionError> {
        MavConnection::send(self.as_ref(), &Default::default(), msg)
            .map_err(MavlinkConnectionError::WriteError)
    }

    #[tracing::instrument(skip(self, filter))]
    async fn send_wait<R>(
        self: Arc<Self>,
        msg: &MavMessage,
        timeout: std::time::Duration,
        filter: impl Fn(MavMessage) -> FilterRes<R> + Send + Sync + 'static,
    ) -> Result<Option<R>, MavlinkConnectionError>
    where
        R: Send + Sync + 'static,
    {
        let (tx, mut rx) = tokio::sync::mpsc::channel(1);

        let monitor = self.clone().monitor(Some(timeout), move |msg| {
            if let FilterRes::Ready(inner) = filter(msg) {
                tx.try_send(inner);
                None
            } else {
                Some(())
            }
        });

        self.send(msg)?;

        tracing::event!(tracing::Level::TRACE, "Message Sent");

        rx.recv().await.ok_or(MavlinkConnectionError::Timeout)
    }

    fn monitor(
        self: Arc<Self>,
        timeout: Option<std::time::Duration>,
        // Currently when this returns None it implies that we are done
        // And returning Some(()) means that we should continue monitoring
        // I don't like this API, should at least change to maybe true/false to be more clear
        monitor: impl Fn(MavMessage) -> Option<()> + Send + Sync + 'static,
    ) -> JoinHandle<Result<(), MavlinkConnectionError>> {
        let instant = std::time::Instant::now();
        tokio::task::spawn(async move {
            loop {
                if let Some(t) = timeout {
                    if instant.elapsed() > t {
                        return Err(MavlinkConnectionError::Timeout);
                    }
                }

                match self.clone()._receive().await {
                    Ok((header, msg)) => {
                        if self.validate(header) && monitor(msg).is_none() {
                            return Ok(());
                        }
                    }
                    _ => {}
                }
            }
        })
    }

    // This is in fact a blocking call -- since conn.recv() is blocking --
    // therefore, we cannot really "timeout" on receiving IFF there are NO messages coming through
    // If this function blocks forever, meaning there are no messages to return -- then there is nothing we can do...
    // BUT, since this is in a spawn_blocking, it will at least not hold up the execution of OTHER things
    //  (except things in it's consequent execution tree.
    async fn _receive(self: Arc<Self>) -> Result<(MavHeader, MavMessage), MavlinkConnectionError> {
        return tokio::task::spawn_blocking({
            let conn = self.clone();
            move || conn.recv()
        })
        .await
        .unwrap()
        .map_err(MavlinkConnectionError::ReadError);
    }
}

#[cfg(any(test, feature = "tester"))]
pub mod test {

    use std::{
        fmt::Debug,
        sync::{Arc, Mutex},
    };

    use super::{FilterRes, MavlinkConnection, MavlinkConnectionError};
    use mavlink::{
        ardupilotmega::{MavMessage, HEARTBEAT_DATA},
        MavConnection,
    };

    pub struct TestMavConnection {
        last_value: Arc<Mutex<Option<MavMessage>>>,
        value: Arc<Mutex<Option<MavMessage>>>,
    }

    impl Debug for TestMavConnection {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "TestMavConnection [{:?}]", self.value.lock().unwrap())
        }
    }

    impl TestMavConnection {
        pub fn new() -> Self {
            Self {
                value: Default::default(),
                last_value: Default::default(),
            }
        }
    }

    impl MavConnection<MavMessage> for TestMavConnection {
        fn get_protocol_version(&self) -> mavlink::MavlinkVersion {
            mavlink::MavlinkVersion::V1
        }
        fn set_protocol_version(&mut self, _version: mavlink::MavlinkVersion) {}
        fn send(
            &self,
            _header: &mavlink::MavHeader,
            data: &MavMessage,
        ) -> Result<usize, mavlink::error::MessageWriteError> {
            self.value.lock().unwrap().replace(data.clone());
            // TODO(bjc) this is not representative
            Ok(1)
        }

        fn recv(
            &self,
        ) -> Result<(mavlink::MavHeader, MavMessage), mavlink::error::MessageReadError> {
            loop {
                if let Some(value) = self.value.lock().unwrap().as_mut() {
                    let mut last_value = self.last_value.lock().unwrap();

                    if last_value.as_ref().map(|i| i != value).unwrap_or(true) {
                        last_value.replace(value.clone());
                        return Ok((Default::default(), value.clone()));
                    }
                }
                std::thread::sleep(std::time::Duration::from_secs_f64(0.01));
            }
        }
    }

    fn start_heartbeats(conn: Arc<Box<TestMavConnection>>) {
        tokio::spawn({
            let conn = conn.clone();
            async move {
                let mut int = tokio::time::interval(std::time::Duration::from_secs(1));
                loop {
                    int.tick().await;

                    conn.send(&MavMessage::HEARTBEAT(Default::default()))
                        .unwrap();
                }
            }
        });
    }
    #[tokio::test]
    async fn timeout() {
        let connection = Arc::new(Box::new(TestMavConnection::new()));

        start_heartbeats(connection.clone());

        let res = connection
            .send_wait::<()>(
                &MavMessage::MISSION_ITEM_INT(Default::default()),
                std::time::Duration::from_secs_f64(0.001),
                |_| FilterRes::NotReady,
            )
            .await;

        assert!(matches!(res, Err(MavlinkConnectionError::Timeout)));
    }

    #[tokio::test]
    async fn valid() {
        let connection = Arc::new(Box::new(TestMavConnection::new()));

        let res = connection
            .send_wait(
                &MavMessage::MISSION_ITEM_INT(Default::default()),
                std::time::Duration::from_secs_f64(0.001),
                |_| FilterRes::Ready(Some(1)),
            )
            .await;

        assert!(matches!(res, Ok(Some(1))));
    }

    #[tokio::test]
    async fn invalid() {
        let connection = Arc::new(Box::new(TestMavConnection::new()));

        let res = connection
            .send_wait::<()>(
                &MavMessage::MISSION_ITEM_INT(Default::default()),
                std::time::Duration::from_secs_f64(0.001),
                |_| FilterRes::Ready(None),
            )
            .await;

        assert!(matches!(res, Ok(None)));
    }

    #[tokio::test]
    async fn wait_for_msg() {
        let connection = Arc::new(Box::new(TestMavConnection::new()));

        start_heartbeats(connection.clone());

        let res = connection
            .clone()
            .send_wait(
                &MavMessage::MISSION_ITEM_INT(Default::default()),
                std::time::Duration::from_secs(5),
                |msg| {
                    if let MavMessage::HEARTBEAT(_) = msg {
                        FilterRes::Ready(Some("finished"))
                    } else {
                        FilterRes::NotReady
                    }
                },
            )
            .await;

        assert!(matches!(res, Ok(Some("finished"))))
    }

    #[tokio::test]
    async fn monitor_timeout() {
        let connection = Arc::new(Box::new(TestMavConnection::new()));

        start_heartbeats(connection.clone());

        let res = connection
            .monitor(
                Some(std::time::Duration::from_secs_f64(0.001)),
                |_| Some(()),
            )
            .await
            .unwrap();

        assert!(matches!(res, Err(MavlinkConnectionError::Timeout)))
    }

    #[tokio::test]
    async fn monitor_no_timeout() {
        let connection = Arc::new(Box::new(TestMavConnection::new()));

        start_heartbeats(connection.clone());

        let (tx, mut rx) = tokio::sync::watch::channel(None);

        connection.monitor(None, move |msg| {
            if let MavMessage::HEARTBEAT(data) = msg {
                tx.send(Some(data));
                return None;
            }
            Some(())
        });

        tokio::time::timeout(std::time::Duration::from_secs(2), async move {
            while rx.changed().await.is_ok() {
                assert!(matches!(rx.borrow().clone(), Some(HEARTBEAT_DATA { .. })));
            }
        })
        .await
        .unwrap();
    }
}
