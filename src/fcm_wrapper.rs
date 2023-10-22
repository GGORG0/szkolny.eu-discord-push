use fcm_push_listener::{FcmMessage, FcmPushListener, Registration};
use futures::stream::Stream;
use futures::task::{Context, Poll};
use std::pin::Pin;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::task::Waker;

pub struct FcmMessageStream {
    receiver: Arc<Mutex<mpsc::Receiver<FcmMessage>>>,

    waker: Arc<Mutex<Option<Waker>>>,
}

impl FcmMessageStream {
    pub async fn new(
        registration: Registration,
        received_persistent_ids: Vec<String>,
    ) -> Result<Self, fcm_push_listener::Error> {
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let waker: Arc<Mutex<Option<Waker>>> = Arc::new(Mutex::new(None));
        let waker_clone = waker.clone();

        // Register the message listener here and send messages to the channel
        let mut listener = FcmPushListener::create(
            registration,
            move |message| {
                sender.send(message).unwrap();
                if let Some(waker) = waker_clone.lock().unwrap().take() {
                    waker.wake();
                }
            },
            received_persistent_ids,
        );

        tokio::spawn(async move {
            loop {
                println!("  -> Connecting to FCM...");
                while let Err(e) = listener.connect().await {
                    eprintln!("Failed to connect to FCM: {}", e);
                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                }
            }
        });

        Ok(FcmMessageStream { receiver, waker })
    }
}

impl Stream for FcmMessageStream {
    type Item = FcmMessage;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let receiver = self.receiver.lock().unwrap();
        let waker = cx.waker().clone();
        match receiver.try_recv() {
            Ok(message) => Poll::Ready(Some(message)),
            Err(mpsc::TryRecvError::Empty) => {
                *self.waker.lock().unwrap() = Some(waker);
                Poll::Pending
            }
            Err(mpsc::TryRecvError::Disconnected) => Poll::Ready(None),
        }
    }
}
