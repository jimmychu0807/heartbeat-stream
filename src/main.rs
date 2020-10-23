use tokio::{
	stream::{self, Stream},
	time::{sleep, interval},
};
use future_timer::Delay;
use std::time::Duration;

struct RandomStream {
	min_dur: Duration,
	max_dur: Duration,
	delay: Option<Delay>,
	char: char
};

impl RandomStream {
	pub fn new(min: u64, max: u64, char: char) -> Self {
		let rs = RandomStream {
			min_dur: Duration::from_secs(min),
			max_dur: Duration::from_secs(max),
			delay: None,
			char
		};

		rs.set_next_delay();
		rs
	}

	fn set_next_delay(mut self) {
		self.delay = Some(Delay::new(self.min_dur));
	}
}

impl Stream for RandomStream {
	// TODO: can you enhance on this
	type Item = char;
	fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>>
}

struct BurstingStream {

}


#[tokio::main]
pub async fn main() -> Result<()> {
  println!("Hello, world!");
  Ok()
}
