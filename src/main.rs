// Request-scoped,
// aggregatable events.                                                                 Subscribers
// ┌────────────┐                                                                       ┌───────────────────┐
// │ ┌────────┐ │                                                                       │ ┌───────────────┐ │
// │ │  Span  │ │───┐   Global Queue            Supervisor                         ┌───▶│ │  Prometheus   │ │
// │ └────────┘ │   │   ┌──────────────┐        ┌──────────────────────────────┐   │    │ └───────────────┘ │
// │ ┌────────┐ │   │   │ Insert self  │        │     Pop span from queue,     │   │    │ ┌───────────────┐ │
// │ │  Span  │ │───┼──▶│into queue on ├───────▶│publishing/cloning the span to├───┼───▶│ │    stdout     │ │
// │ └────────┘ │   │   │    drop.     │        │ each registered subscriber.  │   │    │ └───────────────┘ │
// │ ┌────────┐ │   │   └──────────────┘        └──────────────────────────────┘   │    │ ┌───────────────┐ │
// │ │  Span  │ │───┘                                                              └───▶│ │ Zipkin/X-Ray  │ │
// │ └────────┘ │                                                                       │ └───────────────┘ │
// └────────────┘                                                                       └───────────────────┘

use crossbeam::{channel, queue::SegQueue, Receiver, Sender};
use failure::Error;
use lazy_static::lazy_static;

lazy_static! {
    static ref QUEUE: SegQueue<Span> = SegQueue::new();
}

#[derive(Debug, Clone)]
struct Span(String);

impl Drop for Span {
    fn drop(&mut self) {
        QUEUE.push(self.clone());
    }
}

trait Supervisor {
    fn broadcast(&self);
}

type Channels = Vec<(Sender<Span>, Receiver<Span>)>;
struct Super {
    queue: SegQueue<Span>,
    channels: Channels,
}

impl Super {
    fn new(queue: SegQueue<Span>, channels: Channels) -> Self {
        Super { queue, channels }
    }
}

impl Supervisor for Super {
    fn broadcast(&self) {}
}

fn main() -> Result<(), Error> {
    for _ in 0..5 {
        {
            Span("Hello!".to_string());
        }

        match QUEUE.try_pop() {
            Some(s) => println!("{:?}", s),
            None => unreachable!(),
        };
    }

    Ok(())
}
