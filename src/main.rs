/// 
/// Copyright 2023, [object Object]
/// Licensed under MIT
///

use std::error::Error;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use font_kit::family_name::FamilyName;
use font_kit::font::Font;
use font_kit::properties::Properties;
use font_kit::source::SystemSource;
use minifb::{Window, WindowOptions};
use raqote::{DrawOptions, DrawTarget, PathBuilder, Point, SolidSource, Source};
use rodio::source::SineWave;
use rodio::{OutputStream, Sink, Source as OtherSource};

mod sorters;

const ELEMENTS: u32 = 50; // Number of elements to sort
const RESOLUTION: (usize, usize) = (800, 600); // Resolution of the window
const BASE_FREQUENCY: f32 = 880.0; // Base frequency for the lowest element
const BASE_DURATION: f32 = 0.9; // Base duration for the lowest element
const AMPLITUDE: f32 = 0.2; // Amplitude of the sound
const GAP: f32 = 0.1; // Gap between elements

pub struct Iteration {
    arr: Vec<u32>,   // the array being sorted
    index: Vec<u32>, // index(s) of the elements being compared
}

fn sort(arr: &mut [u32]) -> Vec<Iteration> {
    let mut sortable = sorters::SortableArray::new(arr);
    // sortable.merge_sort_rec(0, sortable.arr.len() - 1);
    // sortable.quicksort(0, sortable.arr.len() - 1);
    // sortable.bubble_sort(); // bubble sort is really slow
    // ... etc

    // selection_sort shell heap insertion cocktail
    // sortable.heap_sort();
    sortable.cocktail_sort();
    // sortable.selection_sort();
    // sortable.insertion_sort();
    // sortable.shell_sort();
    // sortable.quicksort(0, sortable.arr.len() - 1);

    // sortable.bogo_sort();

    sortable.get_iterations()
}

fn shuffle(arr: &mut [u32]) {
    use rand::seq::SliceRandom;
    use rand::thread_rng;

    let mut rng = thread_rng();
    arr.shuffle(&mut rng);
}

fn audio_thread(
    audio_rx: Receiver<(f32, f32)>,
    should_stop: Arc<AtomicBool>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();

        loop {
            if let Ok((tone, length)) = audio_rx.recv() {
                play_sine(tone, length, &sink)
            }

            sink.sleep_until_end();

            if should_stop.load(Ordering::Relaxed) {
                break;
            }
        }

        sink.stop();

        drop(stream_handle);
    })
}

fn play_sine(freq: f32, duration: f32, sink: &Sink) {
    let source = SineWave::new(freq)
        .fade_in(Duration::from_secs_f32(duration / ELEMENTS as f32))
        .high_pass(BASE_FREQUENCY as u32)
        .take_duration(Duration::from_secs_f32(duration))
        .amplify(AMPLITUDE);
    sink.append(source);
}

fn get_max(arr: &[u32]) -> u32 {
    let mut max = 0;
    for e in arr {
        if *e > max {
            max = *e;
        }
    }

    max
}

fn draw_and_fill_rectangle(
    i: usize,
    e: &u32,
    iteration: &Iteration,
    duration: f32,
    dt: &mut DrawTarget,
    audio_tx: &Sender<(f32, f32)>,
) {
    let mut pb = PathBuilder::new();


    let element_ratio = *e as f32 / ELEMENTS as f32;
    let height = RESOLUTION.1 as f32 * element_ratio;
    let draw_x = RESOLUTION.0 as f32 / ELEMENTS as f32 * i as f32;
    let draw_y = RESOLUTION.1 as f32 - height; // Subtract height from total height
    let width = (RESOLUTION.0 as f32 / ELEMENTS as f32) - GAP;

    // Draw a square for each element
    pb.rect(draw_x, draw_y, width, height);

    let path = pb.finish();

    if iteration.index.contains(e) {
        dt.fill(
            &path,
            &Source::Solid(SolidSource::from_unpremultiplied_argb(0xff, 234, 82, 82)),
            &DrawOptions::new(),
        );

        let max_index = get_max(&iteration.index);

        if max_index == *e {
            let freq = BASE_FREQUENCY * element_ratio;

            audio_tx.send((freq, duration)).unwrap();
        }
    } else {
        dt.fill(
            &path,
            &Source::Solid(SolidSource::from_unpremultiplied_argb(0xff, 0, 119, 233)),
            &DrawOptions::new(),
        );
    }
}

fn handle_key_input(window: &Window, started: &mut bool) -> bool {
    if window.is_key_down(minifb::Key::Escape) {
        *started = false;

        return true;
    }

    if window.is_key_down(minifb::Key::Space) {
        *started = true;
    }

    false
}

fn draw_text_upper_left(message: &str, dt: &mut DrawTarget, font: &Font) {
    dt.draw_text(
        font,
        36.,
        message,
        Point::new(0., 100.),
        &Source::Solid(SolidSource::from_unpremultiplied_argb(0xff, 0, 0, 0)),
        &DrawOptions::new(),
    );
}

fn update_window(window: &mut Window, dt: &DrawTarget) {
    window
        .update_with_buffer(dt.get_data(), RESOLUTION.0, RESOLUTION.1)
        .unwrap();
}

fn process_iteration(
    iterations: &Vec<Iteration>,
    num_iterations: &mut usize,
    duration: f32,
    dt: &mut DrawTarget,
    audio_tx: &Sender<(f32, f32)>,
) {
    if let Some(iteration) = iterations.get(*num_iterations) {
        for (i, e) in iteration.arr.iter().enumerate() {
            draw_and_fill_rectangle(i, e, iteration, duration, dt, audio_tx);
        }

        *num_iterations += 1;
    } else {
        std::thread::sleep(std::time::Duration::from_millis(1000));

        *num_iterations = 0;
    }
}

// Main loop!
fn main_loop(
    window: &mut Window,
    dt: &mut DrawTarget,
    font: &Font,
    iterations: &Vec<Iteration>,
    audio_tx: &Sender<(f32, f32)>,
) -> Result<(), Box<dyn Error>> {
    let mut num_iterations = 0;
    let mut started = false;
    loop {
        let duration = BASE_DURATION / ELEMENTS as f32;

        // Background
        dt.clear(SolidSource::from_unpremultiplied_argb(
            0xff, 0xf0, 0xff, 0xff,
        ));

        let should_end = handle_key_input(window, &mut started);

        if should_end {
            // Send a dummy value to the audio thread to update it so it stops.
            audio_tx.send((0.0, 0.0)).unwrap();
            
            break;
        }

        if !started {
            draw_text_upper_left("Press space to start!", dt, font);

            update_window(window, dt);

            continue;
        }

        process_iteration(iterations, &mut num_iterations, duration, dt, audio_tx);

        update_window(window, dt);

        std::thread::sleep(Duration::from_secs_f32(duration));
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (audio_tx, audio_rx) = channel::<(f32, f32)>();

    // Start the audio thread
    let thread_should_stop = Arc::new(AtomicBool::new(false));
    let handle = audio_thread(audio_rx, thread_should_stop.clone());

    let mut window = Window::new(
        "Sorting Visualizer",
        RESOLUTION.0,
        RESOLUTION.1,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    let font = SystemSource::new()
        .select_best_match(&[FamilyName::SansSerif], &Properties::new())
        .unwrap()
        .load()
        .unwrap();

    let mut dt = DrawTarget::new(RESOLUTION.0 as i32, RESOLUTION.1 as i32);

    let mut iterations = (1..ELEMENTS + 1).collect::<Vec<u32>>();

    shuffle(&mut iterations);
    let iterations = sort(&mut iterations);

    main_loop(&mut window, &mut dt, &font, &iterations, &audio_tx)?;

    thread_should_stop.store(true, Ordering::Relaxed);

    handle.join().unwrap();
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sort() {
        let mut arr = [1, 5, 2, 3, 4];
        sort(&mut arr);

        assert_eq!(arr, [1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_shuffle() {
        let mut arr: [u32; 5] = [1, 2, 3, 4, 5];
        shuffle(&mut arr);

        assert_ne!(arr, [1, 2, 3, 4, 5]);
    }
}
