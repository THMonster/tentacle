use futures::{
    channel::{mpsc, oneshot},
    sink::SinkExt,
};

use crate::{error::Error, stream::StreamHandle};

pub(crate) enum Command<T> {
    OpenStream(oneshot::Sender<Result<StreamHandle, Error>>),
    Shutdown(oneshot::Sender<()>),
    AddStream(T),
    CloseOldestStream,
    GetStreamsNum(oneshot::Sender<usize>),
}

/// A session control is used to open the stream or close the session
#[derive(Clone)]
pub struct Control<T>(mpsc::Sender<Command<T>>);

impl<T> Control<T> {
    pub(crate) fn new(sender: mpsc::Sender<Command<T>>) -> Self {
        Control(sender)
    }

    /// Open a new stream to remote session
    pub async fn open_stream(&mut self) -> Result<StreamHandle, Error> {
        let (tx, rx) = oneshot::channel();
        self.0
            .send(Command::OpenStream(tx))
            .await
            .map_err(|_| Error::SessionShutdown)?;
        rx.await.map_err(|_| Error::SessionShutdown)?
    }

    /// shutdown is used to close the session and all streams.
    pub async fn close(&mut self) {
        if self.0.is_closed() {
            return;
        }
        let (tx, rx) = oneshot::channel();
        let _ignore = self.0.send(Command::Shutdown(tx)).await;
        let _ignore = rx.await;
    }

    pub async fn add_stream(&mut self, raw_socket: T) -> Result<(), Error> {
        self.0
            .send(Command::AddStream(raw_socket))
            .await
            .map_err(|_| Error::SessionShutdown)?;
        Ok(())
    }

    pub async fn close_oldest_stream(&mut self) -> Result<(), Error> {
        self.0
            .send(Command::CloseOldestStream)
            .await
            .map_err(|_| Error::SessionShutdown)?;
        Ok(())
    }

    pub async fn get_streams_num(&mut self) -> Result<usize, Error> {
        let (tx, rx) = oneshot::channel();
        self.0
            .send(Command::GetStreamsNum(tx))
            .await
            .map_err(|_| Error::SessionShutdown)?;
        rx.await.map_err(|_| Error::SessionShutdown)
    }
}
