
use std::thread;
use std::sync::mpsc;
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::sync::Mutex;
use quick_js::{Context, JsValue};

type Job = Box<dyn FnOnce(JsValue) + Send + 'static>;

pub struct ThreadPool {
	workers: Vec<Worker>,
    sender: mpsc::Sender<Job>,
}

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(size);
        for id in 0..size {
            workers.push(Worker::new(
                id,
                Arc::clone(&receiver),
                Context::new().unwrap(),
            ));
        }
        ThreadPool { workers, sender }
    }
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce(JsValue) + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.send(job).unwrap();
    }
}

struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>,
    ctx: Context,
}

impl Worker {
   fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>, ctx: Context) -> Worker {
        let thread = thread::spawn(move || {
            let ctx = Context::new().unwrap();
            let shared_ctx = Arc::new(Mutex::new(&ctx));
            let _loaded_renderer = ctx.eval("function renderer (str) { return `${str} is rendered`; }").unwrap();
            let (tx, rx): (mpsc::Sender<Context>, mpsc::Receiver<Context>) = channel();
            let (shared_ctx, tx) = (Arc::clone(&shared_ctx), tx.clone());
            loop {
                let mut ctx = shared_ctx.lock().unwrap();
                let job = receiver.lock().unwrap().recv().unwrap();
                println!("Worker {} got a job; executing.", id);
                let result = ctx.eval("renderer('foobar')").unwrap();
                job(result);
            }
        });

        Worker { id, thread, ctx }
    }
}