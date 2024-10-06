use anyhow::{anyhow, Result};
use rust_music_theory::note::{Note, PitchClass};

pub enum Tuning {
    EqualTemperament,
}

pub fn from_string(string: &str) -> Result<Note> {
    let mut pitch = "".to_string();
    let mut oct = "".to_string();
    for c in string.chars() {
        if c.is_numeric() {
            oct.push(c);
        } else {
            pitch.push(c);
        }
    }

    let n =
        PitchClass::from_str(&pitch).ok_or(anyhow!("can't convert {pitch:?} to pitch class"))?;
    let o = oct
        .parse::<u8>()
        .ok()
        .ok_or(anyhow!("can't convert {oct:?} to octave"))?;

    Ok(Note::new(n, o))
}

pub fn from_note_nr(nr: u8) -> Note {
    println!("nr: {nr}");
    let pitch_class = PitchClass::from_u8(nr % 12);
    let octave = nr / 12;
    Note::new(pitch_class, octave)
}

pub fn from_freq(freq: f32, tuning: Tuning) -> Note {
    match tuning {
        Tuning::EqualTemperament => {
            let a440 = to_note_nr(Note::new(PitchClass::from_str("A").unwrap(), 4));
            from_note_nr(((12.0 * (freq / 440.0).log2()) as i16 + a440 as i16) as u8)
        }
    }
}

pub fn to_note_nr(note: Note) -> u8 {
    note.pitch_class.into_u8() + 12 * note.octave
}

pub fn to_freq(note: Note, tuning: Tuning) -> f32 {
    match tuning {
        Tuning::EqualTemperament => {
            let a440 = to_note_nr(Note::new(PitchClass::from_str("A").unwrap(), 4));
            2f32.powf(((to_note_nr(note) as i16 - a440 as i16) as f32) / 12.0) * 440.0
        }
    }
}
