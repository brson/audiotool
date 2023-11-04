use std::sync::{Mutex, Condvar};

static LOCK: Mutex<bool> = Mutex::new(true);
static CONDVAR: Condvar = Condvar::new();

pub fn init() {
    rx::ctrlc::set_handler(ctrlc_handler).expect("ctrlc");
}

pub fn wait() {
    let pending = LOCK.lock().expect("lock");
    let _guard = CONDVAR.wait_while(pending, |pending| *pending).expect("lock");
}

fn ctrlc_handler() {
    let mut pending = LOCK.lock().expect("lock");
    *pending = false;
    CONDVAR.notify_one();
}

