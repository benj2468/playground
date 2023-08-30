use std::{fmt::Debug, sync::Arc};

use mavlink::ardupilotmega::{MavMessage, MavResult, COMMAND_ACK_DATA};

use crate::connection::{MavlinkConnection, MavlinkConnectionError};

pub trait Command {
    fn command<C>(&self, connection: Arc<C>) -> Result<usize, MavlinkConnectionError>
    where
        C: MavlinkConnection + Debug + Send + Sync;

    fn command_monitor<C>(
        &self,
        connection: Arc<C>,
        timeout: Option<std::time::Duration>,
    ) -> tokio::sync::watch::Receiver<Option<COMMAND_ACK_DATA>>
    where
        C: MavlinkConnection + Debug + Send + Sync;

    fn command_retry<C>(
        &self,
        _connection: Arc<C>,
        _timeout: std::time::Duration,
        _retry_max: u8,
    ) -> Result<Option<COMMAND_ACK_DATA>, MavlinkConnectionError>
    where
        C: MavlinkConnection + Debug + Send + Sync,
    {
        unimplemented!("Command Retries are not implemented")
    }
}

impl Command for mavlink::ardupilotmega::COMMAND_INT_DATA {
    fn command<C>(&self, connection: Arc<C>) -> Result<usize, MavlinkConnectionError>
    where
        C: MavlinkConnection + Debug + Send + Sync,
    {
        connection.send(&MavMessage::COMMAND_INT(self.clone()))
    }

    fn command_monitor<C>(
        &self,
        connection: Arc<C>,
        timeout: Option<std::time::Duration>,
    ) -> tokio::sync::watch::Receiver<Option<COMMAND_ACK_DATA>>
    where
        C: MavlinkConnection + Debug + Send + Sync,
    {
        let (tx, rx) = tokio::sync::watch::channel(None);

        connection.clone().monitor(timeout, move |msg| {
            if let MavMessage::COMMAND_ACK(data) = msg {
                tx.send(Some(data.clone())).unwrap();
                if !matches!(data.result, MavResult::MAV_RESULT_IN_PROGRESS) {
                    return None;
                }
            }
            Some(())
        });

        connection
            .send(&MavMessage::COMMAND_INT(self.clone()))
            .expect("Could not send message across mavlink connection -- must have closed");

        rx
    }
}

impl Command for mavlink::ardupilotmega::COMMAND_LONG_DATA {
    fn command<C>(&self, connection: Arc<C>) -> Result<usize, MavlinkConnectionError>
    where
        C: MavlinkConnection + Debug + Send + Sync,
    {
        connection.send(&MavMessage::COMMAND_LONG(self.clone()))
    }

    fn command_monitor<C>(
        &self,
        connection: Arc<C>,
        timeout: Option<std::time::Duration>,
    ) -> tokio::sync::watch::Receiver<Option<COMMAND_ACK_DATA>>
    where
        C: MavlinkConnection + Debug + Send + Sync,
    {
        let (tx, rx) = tokio::sync::watch::channel(None);

        connection.clone().monitor(timeout, move |msg| {
            if let MavMessage::COMMAND_ACK(data) = msg {
                tx.send(Some(data.clone())).unwrap();
                if !matches!(data.result, MavResult::MAV_RESULT_IN_PROGRESS) {
                    return None;
                }
            }
            Some(())
        });

        connection
            .send(&MavMessage::COMMAND_LONG(self.clone()))
            .expect("Could not send message across mavlink connection -- must have closed");

        rx
    }
}

#[cfg(test)]
mod test {
    use crate::command::Command;
    use std::sync::Arc;

    use mavlink::ardupilotmega::{
        MavMessage, MavResult, COMMAND_ACK_DATA, COMMAND_INT_DATA, COMMAND_LONG_DATA,
    };

    use crate::connection::test::*;

    #[tokio::test]
    async fn command_int() {
        let connection: Arc<Box<TestMavConnection>> = Default::default();

        start_heartbeats(connection.clone());

        COMMAND_INT_DATA::default()
            .command(connection.clone())
            .unwrap();

        assert!(matches!(
            connection.last_sent().unwrap(),
            MavMessage::COMMAND_INT(COMMAND_INT_DATA { .. })
        ));
    }

    #[tokio::test]
    async fn command_long() {
        let connection: Arc<Box<TestMavConnection>> = Default::default();

        start_heartbeats(connection.clone());

        COMMAND_LONG_DATA::default()
            .command(connection.clone())
            .unwrap();

        assert!(matches!(
            connection.last_sent().unwrap(),
            MavMessage::COMMAND_LONG(COMMAND_LONG_DATA { .. })
        ));
    }

    #[tokio::test]
    async fn monitor_int() {
        let connection: Arc<Box<TestMavConnection>> = Default::default();

        let mut rx = COMMAND_INT_DATA::default()
            .command_monitor(connection.clone(), Some(std::time::Duration::from_secs(1)));

        connection.inject_msg(MavMessage::COMMAND_ACK(COMMAND_ACK_DATA {
            result: MavResult::MAV_RESULT_ACCEPTED,
            ..Default::default()
        }));

        tokio::time::timeout(std::time::Duration::from_secs(1), async move {
            while rx.changed().await.is_ok() {
                assert!(matches!(
                    rx.borrow().clone().unwrap(),
                    COMMAND_ACK_DATA {
                        result: MavResult::MAV_RESULT_ACCEPTED,

                        ..
                    }
                ));
            }
        })
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn monitor_long() {
        let connection: Arc<Box<TestMavConnection>> = Default::default();

        let mut rx = COMMAND_LONG_DATA::default()
            .command_monitor(connection.clone(), Some(std::time::Duration::from_secs(1)));

        connection.inject_msg(MavMessage::COMMAND_ACK(COMMAND_ACK_DATA {
            result: MavResult::MAV_RESULT_ACCEPTED,
            ..Default::default()
        }));

        tokio::time::timeout(std::time::Duration::from_secs(1), async move {
            while rx.changed().await.is_ok() {
                assert!(matches!(
                    rx.borrow().clone().unwrap(),
                    COMMAND_ACK_DATA {
                        result: MavResult::MAV_RESULT_ACCEPTED,

                        ..
                    }
                ));
            }
        })
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn monitor_int_failed() {
        let connection: Arc<Box<TestMavConnection>> = Default::default();

        let mut rx = COMMAND_INT_DATA::default().command_monitor(connection.clone(), None);

        connection.inject_msg(MavMessage::COMMAND_ACK(COMMAND_ACK_DATA {
            result: MavResult::MAV_RESULT_FAILED,
            ..Default::default()
        }));

        tokio::time::timeout(std::time::Duration::from_secs(3), async move {
            while rx.changed().await.is_ok() {
                assert!(matches!(
                    rx.borrow_and_update().clone().unwrap(),
                    COMMAND_ACK_DATA {
                        result: MavResult::MAV_RESULT_FAILED,

                        ..
                    }
                ));
            }
            // This tells us that the transmitter has been dropped -- and the monitor is over.
            assert!(rx.has_changed().is_err());
        })
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn monitor_long_failed() {
        let connection: Arc<Box<TestMavConnection>> = Default::default();

        let mut rx = COMMAND_LONG_DATA::default().command_monitor(connection.clone(), None);

        connection.inject_msg(MavMessage::COMMAND_ACK(COMMAND_ACK_DATA {
            result: MavResult::MAV_RESULT_FAILED,
            ..Default::default()
        }));

        tokio::time::timeout(std::time::Duration::from_secs(3), async move {
            while rx.changed().await.is_ok() {
                assert!(matches!(
                    rx.borrow_and_update().clone().unwrap(),
                    COMMAND_ACK_DATA {
                        result: MavResult::MAV_RESULT_FAILED,
                        ..
                    }
                ));
            }
            // This tells us that the transmitter has been dropped -- and the monitor is over.
            assert!(rx.has_changed().is_err());
        })
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn monitor_long_progress() {
        let connection: Arc<Box<TestMavConnection>> = Default::default();

        let mut rx = COMMAND_INT_DATA::default().command_monitor(connection.clone(), None);

        connection.inject_msg(MavMessage::COMMAND_ACK(COMMAND_ACK_DATA {
            result: MavResult::MAV_RESULT_IN_PROGRESS,
            ..Default::default()
        }));

        tokio::time::timeout(std::time::Duration::from_secs(3), async move {
            rx.changed().await.unwrap();

            assert!(matches!(
                rx.borrow().clone().unwrap(),
                COMMAND_ACK_DATA {
                    result: MavResult::MAV_RESULT_IN_PROGRESS,

                    ..
                }
            ));
            // This tells us that the transmitter is still open, and the watch is still happening.
            assert!(rx.has_changed().is_ok());

            connection.inject_msg(MavMessage::COMMAND_ACK(COMMAND_ACK_DATA {
                result: MavResult::MAV_RESULT_ACCEPTED,
                ..Default::default()
            }));

            rx.changed().await.unwrap();
            assert!(matches!(
                rx.borrow().clone().unwrap(),
                COMMAND_ACK_DATA {
                    result: MavResult::MAV_RESULT_ACCEPTED,

                    ..
                }
            ));
            // This tells us that the transmitter is has been dropped, and the watch is over.
            assert!(rx.has_changed().is_err());
        })
        .await
        .unwrap();
    }
}
