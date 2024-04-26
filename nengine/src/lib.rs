//!
//! NEngine Engine
//! 

use std::sync::atomic::Ordering;
use std::thread;
use std::sync::{Arc, RwLock, atomic::AtomicBool};
use std::collections::BinaryHeap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::fmt::Debug;

use threadpool::ThreadPool;

mod system_wrapper;
use system_wrapper::SystemWrapper;

mod renderer;
pub use renderer::Renderer;

/// The basic Engine schedules systems to run at given time intervals in a
/// a threadpool with a singular thread reserved for UI rendering at a given frame rate
pub struct Engine<WORLD, E> {
    // Target frame rate in frames / second
    pub target_frame_rate: u32,
    // The World
    pub world: Arc<RwLock<WORLD>>,

    // Renderer
    renderer: Option<Box<dyn Renderer<WORLD, Error=E>>>,
    // Threadpool
    pool: ThreadPool,
    // Scheduling Queue (Binary Heap)
    scheduling_queue: BinaryHeap<SystemWrapper<WORLD>>,
}

impl<WORLD: Send + Sync + 'static, E: Debug + 'static> Engine<WORLD, E> {
    /// Create a new engine with workers, a target frame rate, a world, and
    /// the systems to act on the world (and their update rate (in us))
    pub fn new(
        frame_rate: u32,
        workers: usize,
        world: Arc<RwLock<WORLD>>,
        mut systems: Vec<(fn(Arc<RwLock<WORLD>>), u128)>,
        renderer: Box<dyn Renderer<WORLD, Error=E>>,
    ) -> Self {
        if workers < 2 {
            panic!("NEnginer Requires at least 2 Threads to Execute");
        }

        let mut scheduling_queue = BinaryHeap::new();
        for (system, update_rate) in systems.drain(..) {
            scheduling_queue.push(SystemWrapper{
                system,
                update_rate,
                priority: 0,
            })
        }

        Self {
            target_frame_rate: frame_rate,
            world,
            pool: ThreadPool::new(workers - 1),
            scheduling_queue,
            renderer: Some(renderer),
        }
    }

    /// Run the Executor
    pub fn run(&mut self) {
        let start_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros();

        let c_world = self.world.clone();
        let frame_delay = 1_000_000 / self.target_frame_rate;

        let running = Arc::new(AtomicBool::new(true));
        let c_running = running.clone();
        let cc_running = running.clone();

        ctrlc::set_handler(move || {
            cc_running.store(false, Ordering::SeqCst);
        }).expect("Unable to Set Ctrl-C Handler");

        let mut renderer = self.renderer.take().unwrap();
        let render_thread_handle = thread::spawn(move || {
            let mut last_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros();
            while running.load(Ordering::SeqCst) {
                let cc_world = c_world.clone();
                let current_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros();
                let delta_time = current_time - last_time;

                if frame_delay as u128 > delta_time {
                    let sleep_time = frame_delay as u128 - delta_time;
                    thread::sleep(Duration::from_micros(sleep_time as u64));
                }

                if let Err(err) = renderer.render(cc_world) {
                    running.store(false, Ordering::SeqCst);
                    // TODO: Add Logger
                    println!("Error Occurred in Rendering: {:?}", err);
                    return renderer;
                }

                last_time = current_time;
            }
            return renderer
        });

        while c_running.load(Ordering::SeqCst) {
            let current_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros();
            let delta_time = current_time - start_time;

            let next_system_start_time = self.scheduling_queue.peek().unwrap().priority;
            if delta_time > next_system_start_time {
                let sleep_time = next_system_start_time.saturating_sub(current_time);
                thread::sleep(Duration::from_micros(sleep_time as u64));
            }

            // Get and update the next system
            let mut system_wrapper = self.scheduling_queue.pop().unwrap();
            let c_world = self.world.clone();
            self.pool.execute(move || (system_wrapper.system)(c_world));
            system_wrapper.priority += system_wrapper.update_rate;
            self.scheduling_queue.push(system_wrapper);
        }

        let renderer = match render_thread_handle.join() {
            Ok(renderer) => renderer,
            Err(err) => panic!("Error Joining Render Thread Handle: {:?}", err),
        };

        self.renderer.replace(renderer);
    }
}
