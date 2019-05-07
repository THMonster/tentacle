use futures::Future;
use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use crate::{
    context::SessionContext,
    error::Error,
    multiaddr::Multiaddr,
    service::{DialProtocol, TargetSession},
    ProtocolId, SessionId,
};
use bytes::Bytes;

/// Error generated by the Service
#[derive(Debug)]
pub enum ServiceError {
    /// When dial remote error
    DialerError {
        /// Remote address
        address: Multiaddr,
        /// error
        error: Error,
    },
    /// When listen error
    ListenError {
        /// Listen address
        address: Multiaddr,
        /// error
        error: Error,
    },
    /// Protocol select fail
    ProtocolSelectError {
        /// Protocol name, if none, timeout or other net problem,
        /// if Some, don't support this proto
        proto_name: Option<String>,
        /// Session context
        session_context: Arc<SessionContext>,
    },
    /// Protocol error during interaction
    ProtocolError {
        /// Session id
        id: SessionId,
        /// Protocol id
        proto_id: ProtocolId,
        /// Codec error
        error: Error,
    },
    /// After initializing the connection, the session does not open any protocol,
    /// suspected fd attack
    SessionTimeout {
        /// Session context
        session_context: Arc<SessionContext>,
    },
    /// Multiplex protocol error
    MuxerError {
        /// Session context
        session_context: Arc<SessionContext>,
        /// error, such as `InvalidData`
        error: Error,
    },
    /// Protocol handle error, will cause memory leaks/abnormal CPU usage
    ProtocolHandleError {
        /// Error message
        error: Error,
        /// Protocol id
        proto_id: ProtocolId,
    },
}

/// Event generated by the Service
#[derive(Debug)]
pub enum ServiceEvent {
    /// A session close
    SessionClose {
        /// Session context
        session_context: Arc<SessionContext>,
    },
    /// A session open
    SessionOpen {
        /// Session context
        session_context: Arc<SessionContext>,
    },
    /// Listen close
    ListenClose {
        /// Listen address
        address: Multiaddr,
    },
    /// Listen start
    ListenStarted {
        /// Listen address
        address: Multiaddr,
    },
}

/// Event generated by all protocol
#[derive(Debug)]
pub enum ProtocolEvent {
    /// Protocol open event
    Connected {
        /// session context
        session_context: Arc<SessionContext>,
        /// Protocol id
        proto_id: ProtocolId,
        /// Protocol version
        version: String,
    },
    /// Received protocol data
    Received {
        /// Session context
        session_context: Arc<SessionContext>,
        /// Protocol id
        proto_id: ProtocolId,
        /// Protocol version
        data: bytes::Bytes,
    },
    /// Protocol close event
    Disconnected {
        /// Protocol id
        proto_id: ProtocolId,
        /// session context
        session_context: Arc<SessionContext>,
    },
    /// Service-level notify
    ProtocolNotify {
        /// Protocol id
        proto_id: ProtocolId,
        /// token
        token: u64,
    },
    /// Session-level notify task
    ProtocolSessionNotify {
        /// Session context
        session_context: Arc<SessionContext>,
        /// Protocol id
        proto_id: ProtocolId,
        /// Notify token
        token: u64,
    },
}

/// Task received by the Service.
///
/// An instruction that the outside world can send to the service
pub(crate) enum ServiceTask {
    /// Send protocol data task
    ProtocolMessage {
        /// Specify which sessions to send to,
        /// None means broadcast
        target: TargetSession,
        /// protocol id
        proto_id: ProtocolId,
        /// Message sending priority
        priority: Priority,
        /// data
        data: Bytes,
    },
    /// Open specify protocol
    ProtocolOpen {
        /// Session id
        session_id: SessionId,
        /// protocol id
        proto_id: ProtocolId,
    },
    /// Close specify protocol
    ProtocolClose {
        /// Session id
        session_id: SessionId,
        /// protocol id
        proto_id: ProtocolId,
    },
    /// Service-level notify task
    ProtocolNotify {
        /// Protocol id
        proto_id: ProtocolId,
        /// Notify token
        token: u64,
    },
    /// Session-level notify task
    ProtocolSessionNotify {
        /// Session id
        session_id: SessionId,
        /// Protocol id
        proto_id: ProtocolId,
        /// Notify token
        token: u64,
    },
    /// Set service notify task
    SetProtocolNotify {
        /// Protocol id
        proto_id: ProtocolId,
        /// Timer interval
        interval: Duration,
        /// The timer token
        token: u64,
    },
    /// Remove serivce notify task
    RemoveProtocolNotify {
        /// Protocol id
        proto_id: ProtocolId,
        /// The timer token
        token: u64,
    },
    /// Set service notify task
    SetProtocolSessionNotify {
        /// Session id
        session_id: SessionId,
        /// Protocol id
        proto_id: ProtocolId,
        /// Timer interval
        interval: Duration,
        /// The timer token
        token: u64,
    },
    /// Remove serivce notify task
    RemoveProtocolSessionNotify {
        /// Session id
        session_id: SessionId,
        /// Protocol id
        proto_id: ProtocolId,
        /// The timer token
        token: u64,
    },
    /// Future task
    FutureTask {
        /// Future
        task: Box<dyn Future<Item = (), Error = ()> + 'static + Send>,
    },
    /// Disconnect task
    Disconnect {
        /// Session id
        session_id: SessionId,
    },
    /// Dial task
    Dial {
        /// Remote address
        address: Multiaddr,
        /// Dial protocols
        target: DialProtocol,
    },
    /// Listen task
    Listen {
        /// Listen address
        address: Multiaddr,
    },
    /// Shutdown service
    Shutdown(bool),
}

impl fmt::Debug for ServiceTask {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::ServiceTask::*;

        match self {
            ProtocolMessage {
                target,
                proto_id,
                priority,
                data,
            } => write!(
                f,
                "id: {:?}, proto_id: {}, message: {:?}, priority: {:?}",
                target, proto_id, data, priority
            ),
            SetProtocolNotify {
                proto_id, token, ..
            } => write!(f, "set protocol({}) notify({})", proto_id, token),
            RemoveProtocolNotify { proto_id, token } => {
                write!(f, "remove protocol({}) notify({})", proto_id, token)
            }
            SetProtocolSessionNotify {
                session_id,
                proto_id,
                token,
                ..
            } => write!(
                f,
                "set protocol({}) session({}) notify({})",
                proto_id, session_id, token
            ),
            RemoveProtocolSessionNotify {
                session_id,
                proto_id,
                token,
            } => write!(
                f,
                "remove protocol({}) session({}) notify({})",
                proto_id, session_id, token
            ),
            ProtocolNotify { proto_id, token } => {
                write!(f, "protocol id: {}, token: {}", proto_id, token)
            }
            ProtocolSessionNotify {
                session_id,
                proto_id,
                token,
            } => write!(
                f,
                "session id: {}, protocol id: {}, token: {}",
                session_id, proto_id, token
            ),
            FutureTask { .. } => write!(f, "Future task"),
            Disconnect { session_id } => write!(f, "Disconnect session [{}]", session_id),
            Dial { address, .. } => write!(f, "Dial address: {}", address),
            Listen { address } => write!(f, "Listen address: {}", address),
            ProtocolOpen {
                session_id,
                proto_id,
            } => write!(f, "Open session [{}] proto [{}]", session_id, proto_id),
            ProtocolClose {
                session_id,
                proto_id,
            } => write!(f, "Close session [{}] proto [{}]", session_id, proto_id),
            Shutdown(_) => write!(f, "Try close service"),
        }
    }
}

/// Priority for send
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum Priority {
    High,
    Normal,
}

impl Priority {
    #[inline]
    pub fn is_high(self) -> bool {
        match self {
            Priority::High => true,
            Priority::Normal => false,
        }
    }
}
