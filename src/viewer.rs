use image::{RgbaImage,ImageBuffer,Luma};
use iced::widget::image::{Handle,Image};
use iced::widget::Container;
use iced::{subscription,Application,Command, Element, Settings, Theme,executor,Renderer};
use image::buffer::ConvertBuffer;
use iced::futures::SinkExt;

#[derive(Debug)]
pub enum ViewMessage {
	UpdateImage(ImageBuffer<Luma<u16>,Vec<u16>>)
}

pub struct Viewer {
	current_image:Handle
}

impl Application for Viewer {
    type Message = ViewMessage;
	type Executor = executor::Default;
    type Flags = ();
	type Theme = Theme;
	
	fn new(_flags: ()) -> (Viewer, Command<Self::Message>) {
		//pick up here, start sampling from the camera and wrap the new images in subscriptions using
		//subscription::run
		let image = Handle::from_path("uiowa.png");
		(Viewer{current_image: image},Command::none())
	}

	fn title(&self) -> String {
		String::from("viewer")
	}

	fn view(&self) -> Element<'_, Self::Message, Self::Theme, iced::Renderer>{
		Container::new(Image::new(self.current_image.clone())).into()
	}
	fn update(&mut self,m:ViewMessage) -> Command<Self::Message>{
		match m {
			ViewMessage::UpdateImage(image) => {
				//convert image to RGBA
				let rgba: RgbaImage = image.convert();
				let handle = Handle::from_pixels(rgba.width(),rgba.height(),rgba.as_raw().clone());
				self.current_image = handle;
			}
		}
		Command::none()
	}
	fn subscription(&self) -> subscription::Subscription<Self::Message> {
		let source = crate::C11440_22CUSource::new(0);
		let stream = source.stream(500);
		//I think the point of this is to generate a unique id
		struct SomeWorker;
		subscription::channel(std::any::TypeId::of::<SomeWorker>(), 100,|mut output| async move {
			loop {
				output.send(ViewMessage::UpdateImage(stream.recv().expect("couldn't grab frame"))).await.expect("couldn't send frame in subscription");
			}
		})
	}
}