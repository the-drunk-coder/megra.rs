use crossbeam::channel::Receiver;
use crossbeam::channel::Sender;
use hound;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
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
    write_interval_ms: f64,
}

pub struct CatchHandle {
    pub handle: Option<thread::JoinHandle<()>>,
    pub running: sync::Arc<AtomicBool>,
}

pub struct RecordingControl<const MAX: usize, const NCHAN: usize> {
    pub is_recording: sync::Arc<AtomicBool>, // communicate with the other thread
    pub catch: Option<Catch<MAX, NCHAN>>,
    pub catch_handle: Option<CatchHandle>,
}

pub fn stop_writer_thread(handle: CatchHandle) {
    handle.running.store(false, Ordering::SeqCst);
    handle.handle.unwrap().join().unwrap();
}

pub fn start_writer_thread<const MAX: usize, const NCHAN: usize>(
    catch: Catch<MAX, NCHAN>,
    samplerate: u32,
    path: String,
) -> CatchHandle {
    let write_interval = catch.write_interval_ms;
    let running = sync::Arc::new(AtomicBool::new(true));
    let running2 = running.clone();

    // create the writer thread
    let builder = thread::Builder::new().name("disk_writer_thread".into());

    let handle = Some(
        builder
            .spawn(move || {
                let spec = hound::WavSpec {
                    channels: NCHAN as u16, // record with global number of channels
                    sample_rate: samplerate,
                    bits_per_sample: 32, // 32bit float is fixed
                    sample_format: hound::SampleFormat::Float,
                };

                let mut logical_time = 0.0;
                let start_time = Instant::now();

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

                    let cur = start_time.elapsed().as_secs_f64();
                    let mut diff = cur - logical_time;
                    if diff < 0.0 {
                        diff = 0.0;
                    }
                    logical_time += write_interval;
                    // needs time correction !
                    thread::sleep(Duration::from_secs_f64(write_interval - diff));
                }
            })
            .unwrap(),
    );

    CatchHandle { handle, running }
}

pub fn init_real_time_stream<const MAX: usize, const NCHAN: usize>(
    block_interval_ms: f64,
    write_interval_ms: f64,
) -> (Throw<MAX, NCHAN>, Catch<MAX, NCHAN>) {
    let (tx_send, rx_send): (
        Sender<StreamItem<MAX, NCHAN>>,
        Receiver<StreamItem<MAX, NCHAN>>,
    ) = crossbeam::channel::bounded(2000);

    let (tx_return, rx_return): (
        Sender<StreamItem<MAX, NCHAN>>,
        Receiver<StreamItem<MAX, NCHAN>>,
    ) = crossbeam::channel::bounded(2000);

    // assume write interval is smaller than block interval ...
    // also, use a safety margin
    let pre_fill: usize = ((write_interval_ms / block_interval_ms) * 1.6) as usize;
    println!("real time stream pre-fill {}", pre_fill);
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
        write_interval_ms,
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
        let (throw, catch) = init_real_time_stream::<512, 2>(0.003, 0.1);
        let path = "megra_recording.wav".to_string();
        let handle = start_writer_thread(catch, 44100, path);

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
