use std::{
    thread,
    error::Error,
    sync::{mpsc, Arc, Mutex}
};

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Job>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    /// Create a new ThreadPool
    ///
    /// The size is the number of threads in the pool.
    pub fn new(size: usize) -> Result<ThreadPool, PoolCreationError> {
        if size == 0 {
            return Err(PoolCreationError::new("Number of pools must be greater than zero!"));
        }

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(match Worker::new(id, Arc::clone(&receiver)) {
                Ok(w) => w,
                Err(e) => return Err(PoolCreationError::new(e)),
            });
        }

        Ok(ThreadPool { workers, sender })
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.sender.send(job).unwrap();
    }
}

struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>,
}

impl Worker {
    fn new (id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Result<Worker, std::io::Error>
    {
        let builder = thread::Builder::new();
        let thread = builder.spawn(move || loop {
            let job = receiver.lock().unwrap().recv().unwrap();

            println!("Worker {id} got a job; executing");

            job();
        })?;

        Ok(Worker { id, thread })
    }
}

#[derive(Debug)]
pub struct PoolCreationError {
    inner: Box<dyn Error + Send + Sync>,
}

impl PoolCreationError {
    fn new<E>(inner: E) -> Self
    where
        E: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        PoolCreationError { inner: inner.into() }
    }
}
