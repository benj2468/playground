use std::sync::Arc;

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
        let receiver = tokio::task::spawn({
            let conn = self.clone();
            async move {
                loop {
                    match tokio::task::spawn_blocking({
                        let conn = conn.clone();
                        move || conn.recv()
                    })
                    .await
                    {
                        Ok(Ok((header, msg))) => {
                            if conn.validate(header) {
                                if let FilterRes::Ready(data) = filter(msg) {
                                    return Ok(data);
                                }
                            }
                        }
                        Ok(Err(err)) => return Err(MavlinkConnectionError::ReadError(err)),
                        _ => {}
                    }
                }
            }
        });

        self.send(msg)?;

        tracing::event!(tracing::Level::TRACE, "Message Sent");

        tokio::time::timeout(timeout, receiver)
            .await
            .map_err(|_| MavlinkConnectionError::Timeout)?
            .map_err(|e| MavlinkConnectionError::Other(e.to_string()))?
    }

    fn monitor(
        self: Arc<Self>,
        timeout: Option<std::time::Duration>,
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

                match tokio::task::spawn_blocking({
                    let conn = self.clone();
                    move || conn.recv()
                })
                .await
                {
                    Ok(Ok((header, msg))) => {
                        if self.validate(header) && monitor(msg).is_none() {
                            return Ok(());
                        }
                    }
                    _ => {}
                }
            }
        })
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use super::{FilterRes, MavlinkConnection, MavlinkConnectionError};
    use mavlink::{
        ardupilotmega::{MavMessage, HEARTBEAT_DATA},
        MavConnection,
    };

    pub struct TestMavConnection {
        rx: tokio::sync::watch::Receiver<Option<MavMessage>>,
        tx: tokio::sync::watch::Sender<Option<MavMessage>>,
    }

    impl TestMavConnection {
        fn new() -> Self {
            let (tx, rx) = tokio::sync::watch::channel(None);
            Self { tx, rx }
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
            self.tx.send(Some(data.clone())).unwrap();
            // TODO(bjc) this is not representative
            Ok(1)
        }

        fn recv(
            &self,
        ) -> Result<(mavlink::MavHeader, MavMessage), mavlink::error::MessageReadError> {
            loop {
                if let Some(inner) = self.rx.borrow().clone() {
                    return Ok((Default::default(), inner));
                }
                std::thread::sleep(std::time::Duration::from_secs_f64(0.001));
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
            .monitor(Some(std::time::Duration::from_secs_f64(0.001)), |_| Some(()))
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
