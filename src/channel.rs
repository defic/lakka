
// Expand here with feature flags for alternative channels, such as kanal

pub mod mpsc {
    pub type SendError<T> = tokio::sync::mpsc::error::SendError<T>;
    pub type RecvError = tokio::sync::oneshot::error::RecvError;
    pub type Sender<T> = tokio::sync::mpsc::Sender<T>;
    pub type Receiver<T> = tokio::sync::mpsc::Receiver<T>;

    pub fn channel<T>(capacity: usize) -> (Sender<T>, Receiver<T>) {
        tokio::sync::mpsc::channel(capacity)
    }
}
