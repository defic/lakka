// Expand here with feature flags for alternative channels, such as kanal

pub mod mpsc {
    pub type SendError<T> = tokio::sync::mpsc::error::SendError<T>;
    pub type RecvError = tokio::sync::oneshot::error::RecvError;
    pub type Sender<T> = tokio::sync::mpsc::Sender<T>;
    pub type Receiver<T> = tokio::sync::mpsc::Receiver<T>;

    pub type UnboundedSender<T> = tokio::sync::mpsc::UnboundedSender<T>;
    pub type UnboundeReceiver<T> = tokio::sync::mpsc::UnboundedReceiver<T>;

    pub fn channel<T>(capacity: usize) -> (Sender<T>, Receiver<T>) {
        tokio::sync::mpsc::channel(capacity)
    }

    pub fn unbounded_channel<T>() -> (UnboundedSender<T>, UnboundeReceiver<T>) {
        tokio::sync::mpsc::unbounded_channel()
    }
}
