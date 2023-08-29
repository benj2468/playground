use std::{fmt::Debug, sync::Arc};

use mavlink::ardupilotmega::{
    MavMessage, MavMissionResult, MISSION_COUNT_DATA, MISSION_ITEM_INT_DATA,
};
use tracing::instrument;

use crate::connection::{FilterRes, MavlinkConnection, MavlinkConnectionError};

#[derive(Debug, serde::Deserialize)]
pub struct Options {
    mission_count_timeout: std::time::Duration,
    mission_item_timeout: std::time::Duration,
}

#[async_trait::async_trait]
pub trait MissionUpload {
    // If successful, returns the number of mission items uploaded.
    // Otherwise, returns the error.
    async fn upload_mission<C>(
        self,
        connection: Arc<C>,
        options: Options,
    ) -> Result<u16, MissionUploadError>
    where
        C: MavlinkConnection + Debug + Send + Sync;
}

pub enum MissionUploadError {
    TooManyItems(u32),
    Other(String),
    ConnectionError(MavlinkConnectionError),
}

#[async_trait::async_trait]
impl MissionUpload for Vec<MISSION_ITEM_INT_DATA> {
    #[instrument]
    async fn upload_mission<C>(
        self,
        connection: Arc<C>,
        options: Options,
    ) -> Result<u16, MissionUploadError>
    where
        C: MavlinkConnection + Debug + Send + Sync,
    {
        let count = self.len() as u16;

        let count_request = MavMessage::MISSION_COUNT(MISSION_COUNT_DATA {
            count,
            target_system: connection.target_system(),
            target_component: connection.target_component(),
        });

        let mut req = connection
            .clone()
            .send_wait(count_request, options.mission_count_timeout, |msg| {
                if let MavMessage::MISSION_REQUEST_INT(req) = msg {
                    if req.seq == 0 {
                        FilterRes::Ready(Some(req.seq))
                    } else {
                        FilterRes::Ready(None)
                    }
                } else {
                    FilterRes::NotReady
                }
            })
            .await
            .map_err(MissionUploadError::ConnectionError)?
            .ok_or_else(|| MissionUploadError::Other("Invalid Response".to_string()))?;

        while let Some(item) = self.get(req as usize).cloned() {
            req = connection
                .clone()
                .send_wait(
                    MavMessage::MISSION_ITEM_INT(item),
                    options.mission_item_timeout,
                    move |msg| match msg {
                        MavMessage::MISSION_REQUEST_INT(req) => FilterRes::Ready(Some(req.seq)),
                        MavMessage::MISSION_ACK(ack) => match ack.mavtype {
                            MavMissionResult::MAV_MISSION_ACCEPTED => FilterRes::Ready(Some(count)),
                            _ => FilterRes::Ready(None),
                        },
                        _ => FilterRes::NotReady,
                    },
                )
                .await
                .map_err(MissionUploadError::ConnectionError)?
                .ok_or_else(|| MissionUploadError::Other("Invalid Response".to_string()))?;

            if req == count {
                break;
            }
        }

        Ok(count)
    }
}
