use tokio::{
	stream::{Stream, StreamExt},
};
use futures_timer::Delay;
use core::{
	future::Future,
	pin::Pin,
	task::{Context, Poll},
	time::Duration,
};
use std::io::{self, Write};

struct RandomStream {
	min_dur: Duration,
	max_dur: Duration,
	delay: Option<Delay>,
	char: char
}

impl RandomStream {
	pub fn new(min: u64, max: u64, char: char) -> Self {
		let mut rs = RandomStream {
			min_dur: Duration::from_secs(min),
			max_dur: Duration::from_secs(max),
			delay: None,
			char
		};

		rs.set_next_delay();
		rs
	}

	fn set_next_delay(&mut self) {
		self.delay = Some(Delay::new(self.min_dur));
	}
}

impl Stream for RandomStream {
	// TODO: can you enhance on this to pass by from beginning
	type Item = char;
	fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
		match &mut self.delay {
			None => Poll::Ready(None),
			Some(delay) => {
				match Pin::new(delay).poll(cx) {
					Poll::Pending => Poll::Pending,
					Poll::Ready(_) => {
						// set the delay for next random wakeup time
						self.set_next_delay();
						Poll::Ready(Some(self.char))
					}
				}
			}
		}
	}
}

struct BurstingStream {

}


#[tokio::main]
pub async fn main() -> Result<(), ()> {
  println!("Running main async");
  let mut rs = RandomStream::new(3, 8, 'H');

  // how you poll a stream
  while let Some(c) = rs.next().await {
  	print!("{}", c);
  	io::stdout().flush();
  }

  Ok(())
}
