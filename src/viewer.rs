use futures;
use futures::stream::StreamExt;
use iced::futures::SinkExt;
use iced::widget::image::{Handle, Image};
use iced::widget::Container;
use iced::{executor, subscription, Application, Command, Element, Theme};
use image::buffer::ConvertBuffer;
use image::{ImageBuffer, Luma, RgbaImage};
use std::sync::mpsc::{channel, Sender, TryRecvError};
use std::thread;

type IBuffer = ImageBuffer<Luma<u16>, Vec<u16>>;

#[derive(Debug)]
pub enum ViewMessage {
    UpdateImage(ImageBuffer<Luma<u16>, Vec<u16>>),
}

enum ThreadMessage {
    ChangeConsumer(futures::channel::mpsc::Sender<IBuffer>),
    //KillThread //might want something like this later
}

pub struct Viewer {
    threadtx: Sender<ThreadMessage>,
    current_image: Handle,
}

impl Application for Viewer {
    type Message = ViewMessage;
    type Executor = executor::Default;
    type Flags = ();
    type Theme = Theme;

    fn new(_flags: ()) -> (Viewer, Command<Self::Message>) {
        //Spawn a thread which shoves frames into a sender. We will put a handle to a channel which sends to this
        //thread to allow us to change the destination of the frames without restarting the thread
        let (threadtx, threadrx) = channel::<ThreadMessage>();
        thread::spawn(move || {
            let mut source = crate::C11440_22CUSource::new(0);
            //change the exposure
            source.set_exposure(0.002);
            //wait until we have a consumer of frames to start the stream
            let Ok(ThreadMessage::ChangeConsumer(mut frametx)) = threadrx.recv() else {
                panic!("couldn't get a consumer for frames");
            };
            //start the stream
            let stream = source.stream(500);
            //shove frames until asked to stop
            loop {
                //check to make sure we're using the right channel and that we should keep going
                match threadrx.try_recv() {
                    //new consumer
                    Ok(ThreadMessage::ChangeConsumer(newframetx)) => frametx = newframetx,
                    //time to die
                    Err(TryRecvError::Disconnected) => break,
                    //Ok(ThreadMessage::KillThread) => break, //might want to stop eventually
                    //keep going
                    Err(TryRecvError::Empty) => {}
                }
                //grab a frame and shove it in the pipe
                frametx
                    .try_send(stream.recv().expect("couldn't grab frame"))
                    .expect("couldn't send frame")
            }
        });
        let initial_pixels: Vec<u8> = vec![0, 0, 0, 0];
        let initial_handle = Handle::from_pixels(1, 1, initial_pixels);
        (
            Viewer {
                threadtx,
                current_image: initial_handle,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("viewer")
    }

    fn view(&self) -> Element<'_, Self::Message, Self::Theme, iced::Renderer> {
        Container::new(Image::new(self.current_image.clone())).into()
    }
    fn update(&mut self, m: ViewMessage) -> Command<Self::Message> {
        match m {
            ViewMessage::UpdateImage(image) => {
                //convert image to RGBA
                let handle = buf_to_handle(image);
                self.current_image = handle;
            }
        }
        Command::none()
    }
    fn subscription(&self) -> subscription::Subscription<Self::Message> {
        //I think the point of this is to generate a unique id
        struct SomeWorker;
        //clone our sender so the worker can have a copy
        let threadtx = self.threadtx.clone();
        subscription::channel(
            std::any::TypeId::of::<SomeWorker>(),
            100,
            |mut output| async move {
                //register our existence with the frame grabber
                let (frametx, mut framerx) = futures::channel::mpsc::channel::<IBuffer>(30);
                threadtx
                    .send(ThreadMessage::ChangeConsumer(frametx))
                    .expect("couldn't register with frame grabber");
                loop {
                    output
                        .send(ViewMessage::UpdateImage(framerx.select_next_some().await))
                        .await
                        .expect("couldn't send frame in subscription");
                }
            },
        )
    }
}

///Little helper function for converting a [image::ImageBuffer] into a [iced::image::Handle]
fn buf_to_handle(image: ImageBuffer<Luma<u16>, Vec<u16>>) -> Handle {
    let rgba: RgbaImage = image.convert();
    Handle::from_pixels(rgba.width(), rgba.height(), rgba.as_raw().clone())
}
