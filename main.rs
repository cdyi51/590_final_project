// COMP 590: Christina Yi 730479973

use std::sync::{Arc, Mutex};
use std::thread;
use std::sync::mpsc;
use std::time::Duration;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::atomic::AtomicUsize;

struct Fork;

struct Philosopher {
    name: String,
    left_fork: usize,
    right_fork: usize,
    eat_count: AtomicUsize,
    think_count: AtomicUsize,
}

impl Philosopher {
    fn new(name: &str, left_fork: usize, right_fork: usize) -> Philosopher {
        Philosopher {
            name: name.to_string(),
            left_fork,
            right_fork,
            eat_count: AtomicUsize::new(0),
            think_count: AtomicUsize::new(0),
        }
    }

    fn dine(&self, forks: Arc<Vec<Mutex<Fork>>>, stop_signal: Arc<AtomicBool>) {
        while !stop_signal.load(Ordering::Relaxed) {
            // pick up forks
            let left_fork = forks[self.left_fork].lock().unwrap();
            let right_fork = forks[self.right_fork].lock().unwrap();

            // eating
            println!("{} is eating", self.name);
            self.eat_count.fetch_add(1, Ordering::Relaxed);
            thread::sleep(Duration::from_millis(50));

            // dropping forks
            drop(left_fork);
            drop(right_fork);

            // thinking
            println!("{} is thinking", self.name);
            self.think_count.fetch_add(1, Ordering::Relaxed);
            thread::sleep(Duration::from_millis(50));
        }
    }

    fn report(&self) {
        println!(
            "{}: Eat count: {}, Think count: {}",
            self.name,
            self.eat_count.load(Ordering::Relaxed),
            self.think_count.load(Ordering::Relaxed)
        );
    }
}

fn main() {
    let philosopher_names = vec!["Plato", "Aristotle", "Descartes", "Socrates", "Kant"];

    let philosophers: Vec<_> = philosopher_names
        .iter()
        .enumerate()
        .map(|(i, &name)| Arc::new(Philosopher::new(name, i, (i + 1) % philosopher_names.len())))
        .collect();

    let forks = Arc::new((0..philosopher_names.len()).map(|_| Mutex::new(Fork)).collect::<Vec<_>>());

    let stop_signal = Arc::new(AtomicBool::new(false));

    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        println!("Press Enter to stop: ");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        tx.send(()).unwrap();
    });

    let handles: Vec<_> = philosophers
        .iter()
        .map(|philosopher| {
            let forks = Arc::clone(&forks);
            let stop_signal = Arc::clone(&stop_signal);
            let philosopher = Arc::clone(philosopher);
            thread::spawn(move || philosopher.dine(forks, stop_signal))
        })
        .collect();

    rx.recv().unwrap();
    stop_signal.store(true, Ordering::Relaxed);

    for handle in handles {
        handle.join().unwrap();
    }

    for philosopher in &philosophers {
        philosopher.report();
    }
}

