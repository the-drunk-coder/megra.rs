use crossbeam::channel::Receiver;
use crossbeam::channel::Sender;
use hound;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use std::{sync, thread};

struct StreamItem<const MAX: usize, const NCHAN: usize> {
    buffer: [[f32; MAX]; NCHAN],
    size: usize,
}

pub struct Throw<const MAX: usize, const NCHAN: usize> {
    throw_q: crossbeam::channel::Sender<StreamItem<MAX, NCHAN>>,
    return_q: crossbeam::channel::Receiver<StreamItem<MAX, NCHAN>>,
}

impl<const MAX: usize, const NCHAN: usize> Throw<MAX, NCHAN> {
    pub fn write_samples(&self, block: &[[f32; MAX]; NCHAN], size: usize) {
        match self.return_q.try_recv() {
            Ok(mut stream_item) => {
                if size <= MAX {
                    for ch in 0..NCHAN {
                        for s in 0..size {
                            stream_item.buffer[ch][s] = block[ch][s];
                        }
                    }
                    stream_item.size = size;
                    match self.throw_q.send(stream_item) {
                        Ok(_) => {}
                        Err(_) => {
                            println!("couldn't send streamitem");
                        }
                    }
                }
            }
            Err(_) => {}
        }
    }
}

pub struct Catch<const MAX: usize, const NCHAN: usize> {
    catch_q: crossbeam::channel::Receiver<StreamItem<MAX, NCHAN>>,
    return_q: crossbeam::channel::Sender<StreamItem<MAX, NCHAN>>,
}

pub struct CatchHandle {
    handle: Option<thread::JoinHandle<()>>,
    running: sync::Arc<AtomicBool>,
}

pub fn stop_writer_thread(handle: CatchHandle) {
    handle.running.store(false, Ordering::SeqCst);
    handle.handle.unwrap().join().unwrap();
}

pub fn start_writer_thread<const MAX: usize, const NCHAN: usize>(
    catch: Catch<MAX, NCHAN>,
    write_interval: f64,
) -> CatchHandle {
    let running = sync::Arc::new(AtomicBool::new(true));
    let running2 = running.clone();

    let builder = thread::Builder::new().name("disk_writer_thread".into());

    let handle = Some(
        builder
            .spawn(move || {
                let spec = hound::WavSpec {
                    channels: NCHAN as u16,
                    sample_rate: 44100,
                    bits_per_sample: 32,
                    sample_format: hound::SampleFormat::Float,
                };

                let path: &Path = "megra_recording.wav".as_ref();

                let mut writer = hound::WavWriter::create(path, spec).unwrap();

                while running2.load(Ordering::SeqCst) {
                    for mut stream_item in catch.catch_q.try_iter() {
                        for s in 0..stream_item.size {
                            for ch in 0..NCHAN {
                                writer.write_sample(stream_item.buffer[ch][s]).unwrap();
                            }
                        }
                        stream_item.size = 0;
                        catch.return_q.send(stream_item).unwrap();
                    }

                    // needs time correction !
                    thread::sleep(Duration::from_secs_f64(write_interval));
                }
            })
            .unwrap(),
    );

    CatchHandle { handle, running }
}

pub fn init_real_time_stream<const MAX: usize, const NCHAN: usize>(
    pre_fill: usize,
) -> (Throw<MAX, NCHAN>, Catch<MAX, NCHAN>) {
    let (tx_send, rx_send): (
        Sender<StreamItem<MAX, NCHAN>>,
        Receiver<StreamItem<MAX, NCHAN>>,
    ) = crossbeam::channel::bounded(2000);

    let (tx_return, rx_return): (
        Sender<StreamItem<MAX, NCHAN>>,
        Receiver<StreamItem<MAX, NCHAN>>,
    ) = crossbeam::channel::bounded(2000);

    // pre-fill return queue with specified amount of
    // stream items
    for _ in 0..pre_fill {
        tx_return
            .send(StreamItem::<MAX, NCHAN> {
                buffer: [[0.0; MAX]; NCHAN],
                size: 0,
            })
            .unwrap();
    }

    let throw = Throw::<MAX, NCHAN> {
        throw_q: tx_send,
        return_q: rx_return,
    };

    let catch = Catch::<MAX, NCHAN> {
        catch_q: rx_send,
        return_q: tx_return,
    };

    (throw, catch)
}

// TEST TEST TEST
#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use rand::Rng;

    #[test]
    fn test_real_time_stream() {
        let (throw, catch) = init_real_time_stream::<512, 2>(100);

        let handle = start_writer_thread(catch, 0.1);

        let mut buf: [[f32; 512]; 2] = [[1.0; 512]; 2];

        for _ in 0..100 {
            // fill buffer with noise
            for i in 0..512 {
                buf[0][i] = rand::thread_rng().gen_range(-0.5..0.5);
                buf[1][i] = buf[0][i];
            }
            throw.write_samples(&buf, 512);

            thread::sleep(Duration::from_secs_f64(0.003));
        }

        stop_writer_thread(handle);
    }
}
