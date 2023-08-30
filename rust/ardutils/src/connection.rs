use std::{fmt::Debug, sync::Arc};

use mavlink::{
    ardupilotmega::MavMessage,
    error::{MessageReadError, MessageWriteError},
    MavConnection,
};
use tokio::task::JoinHandle;

#[derive(Debug)]
pub enum MavlinkConnectionError {
    Timeout,
    // TODO(bjc) Remove this dependency
    ReadError(MessageReadError),
    WriteError(MessageWriteError),
    Other(String),
}

pub enum FilterRes<T> {
    Ready(Option<T>),
    NotReady,
}

#[async_trait::async_trait]
pub trait MavlinkConnection<Msg>: Send + Sync + 'static
where
    Msg: Send + Sync,
{
    fn send(&self, msg: &Msg) -> Result<usize, MavlinkConnectionError>;

    async fn send_wait<R>(
        self: Arc<Self>,
        msg: &Msg,
        timeout: std::time::Duration,
        filter: impl Fn(Msg) -> FilterRes<R> + Send + Sync + 'static,
    ) -> Result<Option<R>, MavlinkConnectionError>
    where
        R: Send + Sync + 'static,
    {
        let (tx, mut rx) = tokio::sync::mpsc::channel(1);

        self.clone().monitor(Some(timeout), move |msg| {
            if let FilterRes::Ready(inner) = filter(msg) {
                tx.try_send(inner).unwrap();
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
        monitor: impl Fn(Msg) -> Option<()> + Send + Sync + 'static,
    ) -> JoinHandle<Result<(), MavlinkConnectionError>> {
        let instant = std::time::Instant::now();
        tokio::task::spawn(async move {
            loop {
                if let Some(t) = timeout {
                    if instant.elapsed() > t {
                        return Err(MavlinkConnectionError::Timeout);
                    }
                }

                if let Ok(msg) = self.clone()._receive().await {
                    if monitor(msg).is_none() {
                        return Ok(());
                    }
                }
            }
        })
    }

    async fn _receive(self: Arc<Self>) -> Result<Msg, MavlinkConnectionError>;

    fn target_system(&self) -> u8 {
        env!("MAV_TARGET_SYSTEM").parse().unwrap()
    }
    fn target_component(&self) -> u8 {
        env!("MAV_TARGET_COMPONENT").parse().unwrap()
    }
}

#[async_trait::async_trait]
impl<T> MavlinkConnection<mavlink::ardupilotmega::MavMessage> for Box<T>
where
    T: MavConnection<MavMessage> + Send + Sync + 'static,
{
    #[tracing::instrument(skip(self))]
    fn send(&self, msg: &MavMessage) -> Result<usize, MavlinkConnectionError> {
        MavConnection::send(self.as_ref(), &Default::default(), msg)
            .map_err(MavlinkConnectionError::WriteError)
    }
    // This is in fact a blocking call -- since conn.recv() is blocking --
    // therefore, we cannot really "timeout" on receiving IFF there are NO messages coming through
    // If this function blocks forever, meaning there are no messages to return -- then there is nothing we can do...
    // BUT, since this is in a spawn_blocking, it will at least not hold up the execution of OTHER things
    //  (except things in it's consequent execution tree.
    async fn _receive(self: Arc<Self>) -> Result<MavMessage, MavlinkConnectionError> {
        return tokio::task::spawn_blocking({
            let conn = self.clone();
            move || conn.recv()
        })
        .await
        .unwrap()
        .map_err(MavlinkConnectionError::ReadError)
        .map(|r| r.1);
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

    #[derive(Default)]
    pub struct TestMavConnection {
        sent: Arc<Mutex<Option<MavMessage>>>,
        value: Arc<Mutex<Option<MavMessage>>>,
    }

    impl TestMavConnection {
        pub fn inject_msg(&self, data: MavMessage) {
            self.value.lock().unwrap().replace(data);
        }

        pub fn last_sent(&self) -> Option<MavMessage> {
            self.sent.lock().unwrap().take()
        }
    }

    impl Debug for TestMavConnection {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "TestMavConnection [{:?}]", self.value.lock().unwrap())
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
            self.sent.lock().unwrap().replace(data.clone());
            // TODO(bjc) this is not representative
            Ok(1)
        }

        fn recv(
            &self,
        ) -> Result<(mavlink::MavHeader, MavMessage), mavlink::error::MessageReadError> {
            loop {
                if let Some(value) = self.value.lock().unwrap().as_mut().take() {
                    return Ok((Default::default(), value.clone()));
                }
                std::thread::sleep(std::time::Duration::from_secs_f64(0.01));
            }
        }
    }

    pub fn start_heartbeats(conn: Arc<Box<TestMavConnection>>) {
        tokio::spawn({
            async move {
                let mut int = tokio::time::interval(std::time::Duration::from_secs(1));
                loop {
                    int.tick().await;

                    conn.inject_msg(MavMessage::HEARTBEAT(Default::default()));
                }
            }
        });
    }
    #[tokio::test]
    async fn timeout() {
        let connection: Arc<Box<TestMavConnection>> = Default::default();

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
        let connection: Arc<Box<TestMavConnection>> = Default::default();

        start_heartbeats(connection.clone());

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
        let connection: Arc<Box<TestMavConnection>> = Default::default();

        start_heartbeats(connection.clone());

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
        let connection: Arc<Box<TestMavConnection>> = Default::default();

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
        let connection: Arc<Box<TestMavConnection>> = Default::default();

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
        let connection: Arc<Box<TestMavConnection>> = Default::default();

        start_heartbeats(connection.clone());

        let (tx, mut rx) = tokio::sync::watch::channel(None);

        connection.monitor(None, move |msg| {
            if let MavMessage::HEARTBEAT(data) = msg {
                tx.send(Some(data)).unwrap();
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
