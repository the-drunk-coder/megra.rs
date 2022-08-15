pub fn load_flac(path: &str, samplerate: f32) -> Option<(usize, f32, u32, Vec<f32>)> {
    let mut sample_buffer: Vec<f32> = Vec::new();

    if let Ok(mut reader) = claxon::FlacReader::open(path) {
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

        Some((
            duration,
            reader.streaminfo().sample_rate as f32,
            reader.streaminfo().channels,
            sample_buffer,
        ))
    } else {
        None
    }
}

pub fn load_wav(path: &str, _: f32) -> Option<(usize, f32, u32, Vec<f32>)> {
    if let Ok(reader) = hound::WavReader::open(path) {
        let duration = reader.duration() / reader.spec().sample_rate * 1000;
        let channels = reader.spec().channels;
        let sr = reader.spec().sample_rate;

        let sample_buffer: Vec<f32> = match reader.spec().sample_format {
            hound::SampleFormat::Float => reader
                .into_samples::<f32>()
                .map(|x| x.unwrap_or(0.0))
                .collect(),
            hound::SampleFormat::Int => {
                let mut convert_buffer = Vec::new();
                // decode to f32
                let max_val = (i32::MAX >> (32 - reader.spec().bits_per_sample)) as f32;
                for sample in reader.into_samples::<i32>() {
                    let s = sample.unwrap() as f32 / max_val;
                    convert_buffer.push(s);
                }
                convert_buffer
            }
        };
        Some((
            duration.try_into().unwrap(),
            sr as f32,
            channels.into(),
            sample_buffer,
        ))
    } else {
        None
    }
}
