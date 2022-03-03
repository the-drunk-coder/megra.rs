use crossbeam::channel::Receiver;
use crossbeam::channel::Sender;

struct StreamItem<const MAX: usize, const NCHAN: usize> {
    buffer: [[f32; MAX]; NCHAN],
    size: usize,
}

pub struct Throw<const MAX: usize, const NCHAN: usize> {
    throw_q: crossbeam::channel::Sender<StreamItem<MAX, NCHAN>>,
    return_q: crossbeam::channel::Receiver<StreamItem<MAX, NCHAN>>,
}

pub struct Catch<const MAX: usize, const NCHAN: usize> {
    catch_q: crossbeam::channel::Receiver<StreamItem<MAX, NCHAN>>,
    return_q: crossbeam::channel::Sender<StreamItem<MAX, NCHAN>>,
}

pub fn init_real_time_stream<const MAX: usize, const NCHAN: usize>(
    pre_fill: usize
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
	tx_return.send(StreamItem::<MAX,NCHAN> {
	    buffer: [[0.0; MAX]; NCHAN],
	    size: 0
	}).unwrap();
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
