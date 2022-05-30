use crate::frame::{Frame, FrameCodec};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_util::codec::Framed;

pub(crate) struct FramedStream<T> {
    pub state: FSState,
    pub pending_frame: Option<Frame>,
    pub inner: Framed<T, FrameCodec>,
}

impl<T> FramedStream<T>
where
    T: AsyncRead + AsyncWrite + Unpin,
{
    pub fn new(raw_stream: T, max_stream_window_size: u32) -> Self {
        let inner = Framed::new(
            raw_stream,
            FrameCodec::default().max_frame_size(max_stream_window_size),
        );
        Self {
            inner,
            state: FSState::Established,
            pending_frame: None,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(crate) enum FSState {
    Established,
    RemoteClosed,
    RemoteClosedLocalClosing,
    LocalClosed,
    LocalClosing,
    LocalClosingHalf,
    Closed,
}
