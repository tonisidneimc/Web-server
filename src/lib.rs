
use std::{
    thread,
    // import multiple-producer, single-consumer structure for communication between threads
    // import Atomically Reference Counted and Mutex structures for shared memory support
    sync::{mpsc, Arc, Mutex}
};

pub struct ThreadPool {
    workers : Vec<Worker>,
    sender : Option<mpsc::Sender<Job>>
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    pub fn new(size : usize) -> ThreadPool {
        assert!(size > 0); // will panic if the size is zero
        
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool {
            workers,
            sender : Some(sender)
        }
    }
    pub fn execute<F>(&self, f : F) where F : FnOnce() + Send + 'static {
        let job = Box::new(f);

        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());
        
        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);
            
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

struct Worker {
    id : usize,
    thread : Option<thread::JoinHandle<()>>
}

impl Worker {
    fn new(id : usize, receiver : Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || {
            
            loop {
                // acquire the mutex, receive a message from the channel
                let message = receiver.lock().unwrap().recv(); 

                match message {
                    Ok(job) => {
                        println!("Worker {id} got a job; executing.");
                        job();
                    }
                    Err(_) => {
                        println!("Worker {id} disconnected; shutting down.");
                        break;
                    }
                }
            }
        });

        Worker {
            id, thread : Some(thread)
        }
    }
}