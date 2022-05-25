pub fn load_flac(path: &str, samplerate: f32) -> (usize, f32, u32, Vec<f32>) {
    let mut sample_buffer: Vec<f32> = Vec::new();
    let mut reader = claxon::FlacReader::open(path).unwrap();

    let mut duration = if let Some(samples) = reader.streaminfo().samples {
        let tmp_dur = 1000.0
            * ((samples as f32 / reader.streaminfo().channels as f32)
                / reader.streaminfo().sample_rate as f32);
        tmp_dur as usize
    } else {
        200
    };

    if reader.streaminfo().sample_rate != samplerate as u32 {
        println!("adapt duration");
        duration =
            (duration as f32 * (reader.streaminfo().sample_rate as f32 / samplerate)) as usize;
    }

    // decode to f32
    let max_val = (i32::MAX >> (32 - reader.streaminfo().bits_per_sample)) as f32;
    for sample in reader.samples() {
        let s = sample.unwrap() as f32 / max_val;
        sample_buffer.push(s);
    }

    (
        duration,
        reader.streaminfo().sample_rate as f32,
        reader.streaminfo().channels,
        sample_buffer,
    )
}
